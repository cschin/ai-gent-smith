#![allow(dead_code)]
#![allow(unused_imports)]

use askama::Template;
use futures_util::Future;
use library_cards::{LibraryCards, LibraryCardsBuilder};

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
//use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::Sender, RwLock};

use serde_json::Value;

use tower_sessions::Session;
use tracing::debug;
use tron_app::{
    tron_components::{
        self, button::TnButtonBuilder, tn_future, TnActionExecutionMethod, TnAsset,
        TnComponentBaseRenderTrait, TnComponentBaseTrait, TnFutureHTMLResponse, TnFutureString,
        TnHtmlResponse,
    },
    AppData, HtmlAttributes,
};
use tron_components::{
    text::TnTextInput, TnButton, TnComponentState, TnComponentValue, TnContext, TnContextBase,
    TnEvent, TnTextArea,
};
//use std::sync::Mutex;
use sqlx::Acquire;
use sqlx::Postgres;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::{collections::HashMap, default, pin::Pin, sync::Arc, task::Context};
mod fsm;
mod library_cards;

use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct AgentConfiguration {
    provider: String,
    model_name: String,
    prompt: String,
    follow_up_prompt: String,
}

static BUTTON: &str = "button";
static CARDS: &str = "cards";

// Function to get the database URL
fn get_database_url() -> String {
    std::env::var("DATABASE_URL").expect("please set the DATABASE_URL environment variable")
}

// Define the static database handle
pub static DB_POOL: Lazy<PgPool> = Lazy::new(|| {
    let database_url = get_database_url();
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .expect("Failed to create database connection pool")
});

// This is the main entry point of the application
// It sets up the application configuration and state
// and then starts the application by calling tron_app::run
#[tokio::main]
async fn main() {
    let ui_action_routes = Router::<Arc<AppData>>::new()
        .route("/agent/:id/:parameter", get(get_agent))
        .route("/agent/create", post(create_agent));

    let app_config = tron_app::AppConfigure {
        cognito_login: true,
        http_only: false,
        api_router: Some(ui_action_routes),
        ..Default::default()
    };
    // set app state
    let app_share_data = AppData::builder(build_context, layout)
        .set_head(include_str!("../../../templates/head.html"))
        .set_html_attributes(r#"lang="en" data-theme="business""#)
        .build();
    tron_app::run(app_share_data, app_config).await
}

// These functions are used to build the application context,
// layout, and event actions respectively
fn build_context() -> TnContext {
    let mut context = TnContextBase::default();

    LibraryCards::builder()
        .init(CARDS.into(), "cards".into(), "active")
        .set_attr("class", "btn btn-sm btn-outline btn-primary flex-1")
        .set_attr("hx-target", "#count")
        .set_attr("hx-swap", "innerHTML show:top")
        .add_to_context(&mut context);

    build_left_panel(&mut context);

    TnContext {
        base: Arc::new(RwLock::new(context)),
    }
}

const USER_SETTING_BTN: &str = "user_setting_btn";
const SHOW_AGENT_LIB_BTN: &str = "show_agent_btn";
const CREATE_AGENT_BTN: &str = "create_agent_btn";
const LOGOUT_BTN: &str = "logout_btn";

fn build_left_panel(ctx: &mut TnContextBase) {
    let attrs = HtmlAttributes::builder()
        .add("class", "btn btn-sm btn-block m-1 min-w-36")
        .add("hx-target", "#workspace")
        .add("hx-trigger", "click")
        .add("hx-swap", "outerHTML show:top")
        .build()
        .attributes;

    TnButton::builder()
        .init(USER_SETTING_BTN.into(), "User's Setting".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_AGENT_LIB_BTN.into(), "Show Agent Library".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(CREATE_AGENT_BTN.into(), "Create a New Agent".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

}

#[derive(Template)] // this will generate the code...
#[template(path = "app_page.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AppPageTemplate {
    cards: String,
    buttons: Vec<String>,
}

fn layout(context: TnContext) -> TnFutureString {
    tn_future! {
        let context_guard = context.read().await;
        let cards = context_guard.get_initial_rendered_string(CARDS).await;
        let mut buttons = Vec::<String>::new();
        for btn in [USER_SETTING_BTN, SHOW_AGENT_LIB_BTN, CREATE_AGENT_BTN] {
            buttons.push(context_guard.get_rendered_string(btn).await);
        }
        let html = AppPageTemplate { cards, buttons };
        let s = html.render().unwrap();
        println!("{}", s);
        s
    }
}

#[derive(Template)]
#[template(path = "create_simple_agent.html")]
struct SetupAgentTemplate {}

#[derive(Template)]
#[template(path = "update_simple_agent.html")]
struct UpdateAgentTemplate {
    agent_id: i32,
    name: String,
    description: String,
}

#[derive(Template)]
#[template(path = "user_settings.html")]
struct UserSettingsTemplate {
    username: String,
    email: String,
    anthropic_api_key: String,
    openai_api_key: String,
}

fn change_workspace(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {

        tracing::info!(target: "tron_app", "{:?}", event);

        let output: Option<String> = match event.e_trigger.as_str() {

            SHOW_AGENT_LIB_BTN => {
                let context_guard = context.read().await;
                let cards = context_guard.get_initial_rendered_string(CARDS).await;
                Some(cards)
            },

            CREATE_AGENT_BTN => {
                let template = SetupAgentTemplate {};
                Some(template.render().unwrap())
            },

            USER_SETTING_BTN
                 => {
                    let context_guard = context.read().await;
                    if let Some(user_data) = context_guard.get_user_data().await {
                        let username = user_data.username;
                        let email = user_data.email;
                        let template = UserSettingsTemplate {
                            username,
                            email,
                            anthropic_api_key: "".into(),
                            openai_api_key: "".into()
                        };
                        Some(template.render().unwrap())
                    } else {
                        None
                    }},

            _ => None
        };

        if let Some(output) = output {
            Some((HeaderMap::new(), Html::from(output)))
        } else {
            Some((HeaderMap::new(), Html::from("None".to_string())))
        }
    }
}

fn logout(_context: TnContext, _event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        let mut h = HeaderMap::new();
        h.insert("Hx-Redirect", "/logout".parse().unwrap());
        Some((h, Html::from("".to_string())))
    }
}

async fn get_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path((agent_id, parameter)): Path<(i32, String)>,
    session: Session,
) -> impl IntoResponse {
    println!(
        "in get_agent: agent_id {}, parameter: {}",
        agent_id, parameter
    );
    //println!("payload: {:?}", payload);

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

    let db_pool = &DB_POOL.clone();

    let query = format!(
        "SELECT a.agent_id, a.name, a.description FROM agents a
JOIN users u ON a.user_id = u.user_id
WHERE u.username = '{}' AND a.agent_id = {};",
        user_data.username, agent_id
    );
    let row = sqlx::query(&query)
        .fetch_one(db_pool)
        .await
        .expect("db error");
    let name: String = row.try_get("name").unwrap_or_default();
    let description: String = row.try_get("description").unwrap_or_default();
    println!("agent: {}:{} // {}", agent_id, name, description);

    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    let out_html = if let Some(out_html) = match parameter.as_str() {
        "update" => {
            let template = UpdateAgentTemplate {
                agent_id,
                name,
                description,
            };
            Some(template.render().unwrap())
        }
        _ => None,
    } {
        out_html
    } else {
        format!(r#"<div id="workspace">agent_id: {agent_id}, parameter: {parameter}</div>"#)
    };

    (h, Html::from(out_html))
}


use uuid::Uuid;
async fn create_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Json(payload): Json<Value>
) -> impl IntoResponse {
    println!("payload: {:?}", payload);
    let uuid = Uuid::new_v4();
    Html::from(format!(r#"<p class="py-4">{}"#, uuid))


    
}