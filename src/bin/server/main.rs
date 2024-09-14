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
use std::{collections::HashMap, pin::Pin, sync::Arc, task::Context};

mod library_cards;
mod fsm;

static BUTTON: &str = "button";
static CARDS: &str = "cards";

// This is the main entry point of the application
// It sets up the application configuration and state
// and then starts the application by calling tron_app::run
#[tokio::main]
async fn main() {
    let ui_action_routes =
        Router::<Arc<AppData>>::new().route("/agent/:id/:parameter", get(get_agent));

    let app_config = tron_app::AppConfigure {
        cognito_login: true,
        http_only: false,
        api_router: Some(ui_action_routes),
        ..Default::default()
    };
    // set app state
    let app_share_data = AppData::builder(build_context, layout)
        .set_head(include_str!("../../../templates/head.html"))
        .set_html_attributes(r#"lang="en" data-theme="pastel""#)
        .build();
    tron_app::run(app_share_data, app_config).await
}

// These functions are used to build the application context,
// layout, and event actions respectively
fn build_context() -> TnContext {
    let mut context = TnContextBase::default();

    LibraryCards::builder()
        .init(CARDS.into(), "cards".into())
        .set_attr("class", "btn btn-sm btn-outline btn-primary flex-1")
        .set_attr("hx-target", "#count")
        .set_attr("hx-swap", "innerHTML")
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
        .add("hx-swap", "outerHTML")
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
        .init(CREATE_AGENT_BTN.into(), "Create a New Agent ".into())
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

fn change_workspace(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {

        tracing::info!(target: "tron_app", "{:?}", event);

        let output: Option<String> = match event.e_trigger.as_str() {

            SHOW_AGENT_LIB_BTN => {
                let context_guard = context.read().await;
                let cards = context_guard.get_initial_rendered_string(CARDS).await;
                Some(cards)
            },

            USER_SETTING_BTN | CREATE_AGENT_BTN 
                 => Some(format!(r#"<div id="workspace">{}</div>"#,event.e_trigger)),

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
    println!("agent_id {}, parameter: {}", agent_id, parameter);
    //println!("payload: {:?}", payload);

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.user_data.clone();
    println!("user_data: {:?}", user_data);
    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());
    (h, Html::from(format!(r#"<div id="workspace">agent_id: {agent_id}, parameter: {parameter}</div>"#)))
}
