#![allow(dead_code)]
#![allow(unused_imports)]

mod agent_workspace;
mod embedding_service;
mod library_cards;
mod llm_agent;
mod session_cards;
mod asset_space_plot;

use agent_workspace::*;
use ai_gent_lib::llm_agent::{FSMAgentConfig, FSMAgentConfigBuilder};
use askama::Template;
use asset_space_plot::AssetSpacePlot;
use futures_util::Future;
use library_cards::{LibraryCards, LibraryCardsBuilder};
use session_cards::{SessionCards, SessionCardsBuilder};

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
//use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::Sender, Mutex, RwLock};

use serde_json::{json, Value};

use tower_sessions::Session;
use tracing::debug;
use tron_app::{
    tron_components::{
        self,
        button::TnButtonBuilder,
        chatbox,
        text::{self, update_and_send_textarea_with_context},
        tn_future, TnActionExecutionMethod, TnAsset, TnComponentBaseRenderTrait,
        TnComponentBaseTrait, TnFutureHTMLResponse, TnFutureString, TnHtmlResponse,
        TnServiceRequestMsg,
    },
    AppData, HtmlAttributes,
};
use tron_components::{
    text::TnTextInput, TnButton, TnComponentState, TnComponentValue, TnContext, TnContextBase,
    TnEvent, TnTextArea,
};
//use std::sync::Mutex;
use sqlx::Postgres;
use sqlx::{any::AnyRow, prelude::FromRow, query::Query, Acquire};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::{collections::HashMap, default, fs::File, pin::Pin, sync::Arc, task::Context};

use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct AgentConfiguration {
    provider: String,
    model_name: String,
    prompt: String,
    follow_up_prompt: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SimpleAgentSettingForm {
    name: String,
    description: String,
    // provider: String,
    model_name: String,
    prompt: String,
    follow_up_prompt: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AdvAgentSettingForm {
    name: String,
    description: String,
    // provider: String,
    model_name: String,
    agent_config: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AgentSetting {
    name: String,
    description: String,
    // provider: String,
    model_name: String,
    fsm_agent_config: String,
}

static BUTTON: &str = "button";
static LIBRARY_CARDS: &str = "lib_cards";
static SESSION_CARDS: &str = "session_cards";
static AGENT_WORKSPACE: &str = "agent_workspace";
static ASSET_SPACE_PLOT: &str = "asset_space_plot";

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

pub static SUPPORTED_MODELS: &[(&str, &str)] = &[
    ("gpt-3.5-turbo", "OPENAI_API_KEY"),
    ("gpt-4o", "OPENAI_API_KEY"),
    ("gpt-4o-mini", "OPENAI_API_KEY"),
    ("claude-3-haiku-20240307", "ANTHROPIC_API_KEY"),
    ("claude-3-5-sonnet-20241022", "ANTHROPIC_API_KEY"),
];

// This is the main entry point of the application
// It sets up the application configuration and state
// and then starts the application by calling tron_app::run
#[tokio::main]
async fn main() {
    embedding_service::setup_rag_data().await;

    let ui_action_routes = Router::<Arc<AppData>>::new()
        .route("/agent/create", post(create_basic_agent))
        .route("/agent/create_adv", post(create_adv_agent))
        .route("/agent/{id}/update", post(update_basic_agent))
        .route("/agent/{id}/update_adv", post(update_adv_agent))
        .route("/agent/{id}/use", get(use_agent))
        .route("/agent/{id}/show", get(show_agent_setting))
        .route("/agent/{id}/deactivate", post(deactivate_agent))
        .route("/chat/{id}/delete", get(delete_chat))
        .route("/chat/{id}/show", get(show_chat))
        .route("/asset/{id}/show", get(show_asset))
        .route("/check_user", get(check_user));

    let app_config = tron_app::AppConfigure {
        cognito_login: true,
        http_only: false,
        api_router: Some(ui_action_routes),
        ..Default::default()
    };
    // set app state
    let app_share_data = AppData::builder(build_context, layout)
        .set_head(include_str!("../templates/head.html"))
        .set_html_attributes(r#"lang="en" data-theme="business""#)
        .build();
    tron_app::run(app_share_data, app_config).await
}

// These functions are used to build the application context,
// layout, and event actions respectively
fn build_context() -> TnContext {
    let mut context = TnContextBase::default();

    LibraryCards::builder()
        .init(LIBRARY_CARDS.into(), "cards".into(), "active")
        .set_attr("class", "btn btn-sm btn-outline btn-primary flex-1")
        .add_to_context(&mut context);

    SessionCards::builder()
        .init(SESSION_CARDS.into(), "cards".into(), "active")
        .set_attr("class", "btn btn-sm btn-outline btn-primary flex-1")
        .add_to_context(&mut context);

    AgentWorkSpace::builder()
        .init(
            AGENT_WORKSPACE.into(),
            "agent workspace".into(),
            &mut context,
        )
        .add_to_context(&mut context);

    AssetSpacePlot::builder()
        .init(
            ASSET_SPACE_PLOT.into(),
            "asset_space_plot".into(),
            &mut context,
        )
        .add_to_context(&mut context);

    build_left_panel(&mut context);

    TnContext {
        base: Arc::new(RwLock::new(context)),
    }
}

const USER_SETTING_BTN: &str = "user_setting_btn";
const SHOW_AGENT_LIB_BTN: &str = "show_agent_btn";
const BASIC_AGENT_DESIGN_BTN: &str = "basic_agent_design_btn";
const ADV_AGENT_DESIGN_BTN: &str = "adv_agent_design_btn";
const LOGOUT_BTN: &str = "logout_btn";
const SEARCH_AGENT_BTN: &str = "search_agent_btn";

const SHOW_TODAY_SESSION_BTN: &str = "show_today_sessions_btn";
const SHOW_YESTERDAY_SESSION_BTN: &str = "show_yesterday_sessions_btn";
const SHOW_LAST_WEEK_SESSION_BTN: &str = "show_lastweek_sessions_btn";
const SHOW_ALL_SESSION_BTN: &str = "show_all_sessions_btn";

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
        .init(BASIC_AGENT_DESIGN_BTN.into(), "Create A Basic Agent".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    // TnButton::builder()
    //     .init(SEARCH_AGENT_BTN.into(), "Search Agents".into())
    //     .update_attrs(attrs.clone())
    //     .set_action(TnActionExecutionMethod::Await, change_workspace)
    //     .add_to_context(ctx);

    TnButton::builder()
        .init(
            ADV_AGENT_DESIGN_BTN.into(),
            "Create An Advanced Agent".into(),
        )
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_TODAY_SESSION_BTN.into(), "Today's".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_YESTERDAY_SESSION_BTN.into(), "Since Yesterday".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(
            SHOW_LAST_WEEK_SESSION_BTN.into(),
            "Since The Last Week".into(),
        )
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_ALL_SESSION_BTN.into(), "All".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);
}

#[derive(Template)] // this will generate the code...
#[template(path = "app_page.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AppPageTemplate {
    library_cards: String,
    agent_buttons: Vec<String>,
    sessions_buttons: Vec<String>,
}

fn layout(context: TnContext) -> TnFutureString {
    tn_future! {
        let context_guard = context.read().await;
        let library_cards = context_guard.get_initial_rendered_string(LIBRARY_CARDS).await;
        let mut agent_buttons = Vec::<String>::new();
        for btn in [SHOW_AGENT_LIB_BTN,
                    BASIC_AGENT_DESIGN_BTN,
                    ADV_AGENT_DESIGN_BTN] {
            agent_buttons.push(context_guard.get_rendered_string(btn).await);
        };
        let mut sessions_buttons = Vec::<String>::new();
        for btn in [SHOW_TODAY_SESSION_BTN, SHOW_YESTERDAY_SESSION_BTN,
        SHOW_LAST_WEEK_SESSION_BTN, SHOW_ALL_SESSION_BTN] {
            sessions_buttons.push(context_guard.get_rendered_string(btn).await);
        }
        let html = AppPageTemplate { library_cards, agent_buttons, sessions_buttons };
        let s = html.render().unwrap();
        println!("{}", s);
        s
    }
}

#[derive(Template)]
#[template(path = "create_basic_agent.html", escape = "none")]
struct SetupAgentTemplate {
    model_options: Vec<String>,
}

#[derive(Template)]
#[template(path = "update_basic_agent.html", escape = "none")]
struct UpdateBasicAgentTemplate {
    agent_id: i32,
    name: String,
    description: String,
    model_options: Vec<String>,
    prompt: String,
    follow_up_prompt: String,
}

#[derive(Template)]
#[template(path = "update_fsm_agent.html", escape = "none")]
struct UpdateAdvAgentTemplate {
    agent_id: i32,
    name: String,
    description: String,
    model_options: Vec<String>,
    agent_config: String,
}

#[derive(Template)]
#[template(path = "create_fsm_agent.html", escape = "none")]
struct SetupAdvAgentTemplate {
    model_options: Vec<String>,
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
                let cards = context_guard.get_initial_rendered_string(LIBRARY_CARDS).await;
                Some(cards)
            },

            BASIC_AGENT_DESIGN_BTN => {

                let model_options = SUPPORTED_MODELS.iter().map( | (model_name, _) |
                    format!(r#" <option value="{}">{}</option>"#, model_name, model_name) ).collect::<Vec<String>>();

                let template = SetupAgentTemplate { model_options };
                Some(template.render().unwrap())
            },

            ADV_AGENT_DESIGN_BTN => {
                let model_options = SUPPORTED_MODELS.iter().map( | (model_name, _) |
                    format!(r#" <option value="{}">{}</option>"#, model_name, model_name) ).collect::<Vec<String>>();
                let template = SetupAdvAgentTemplate { model_options };
                Some(template.render().unwrap())
            },

            SEARCH_AGENT_BTN => {
                // TODO: need a chat system to find the right agent
                let context_guard = context.read().await;
                let cards = context_guard.get_initial_rendered_string(LIBRARY_CARDS).await;
                Some(cards)

            },

           SHOW_TODAY_SESSION_BTN => {
                let context_guard = context.read().await;
                let mut assets_guard = context_guard.assets.write().await;
                assets_guard.insert("since_then_in_days".into(), TnAsset::U32(1));
                let cards = context_guard.get_initial_rendered_string(SESSION_CARDS).await;
                Some(cards)
            },

            SHOW_YESTERDAY_SESSION_BTN => {
                let context_guard = context.read().await;
                let mut assets_guard = context_guard.assets.write().await;
                assets_guard.insert("since_then_in_days".into(), TnAsset::U32(2));
                let cards = context_guard.get_initial_rendered_string(SESSION_CARDS).await;
                Some(cards)
            },

            SHOW_LAST_WEEK_SESSION_BTN => {
                let context_guard = context.read().await;
                let mut assets_guard = context_guard.assets.write().await;
                assets_guard.insert("since_then_in_days".into(), TnAsset::U32(7));
                let cards = context_guard.get_initial_rendered_string(SESSION_CARDS).await;
                Some(cards)
            },

            SHOW_ALL_SESSION_BTN => {
                let context_guard = context.read().await;
                let mut assets_guard = context_guard.assets.write().await;
                //assets_guard.remove("since_then_in_days");
                assets_guard.insert("since_then_in_days".into(), TnAsset::U32(3650));
                let cards = context_guard.get_initial_rendered_string(SESSION_CARDS).await;
                Some(cards)
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

            _ => {
                let context_guard = context.read().await;
                let cards = context_guard.get_initial_rendered_string(LIBRARY_CARDS).await;
                Some(cards)
            }
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

struct AgentQueryResult {
    agent_id: i32, // or whatever type agent_id is in your database
    name: String,
    description: Option<String>,
    status: String,                   // or an enum if status is represented as such
    configuration: serde_json::Value, // assuming configuration is stored as JSON
    class: String,                    // or another appropriate type
}

async fn show_agent_setting(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(agent_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    println!("in show_agent: agent_id {}", agent_id);
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

    let db_pool = DB_POOL.clone();

    let row = sqlx::query_as!(
        AgentQueryResult,
        "SELECT a.agent_id, a.name, a.description, a.status, a.configuration, a.class FROM agents a
JOIN users u ON a.user_id = u.user_id
WHERE u.username = $1 AND a.agent_id = $2;",
        user_data.username,
        agent_id
    )
    .fetch_one(&db_pool)
    .await
    .unwrap();
    match row.class.as_str() {
        "basic" => show_basic_agent_setting(&row, agent_id),
        "advanced" => show_adv_agent_setting(&row, agent_id),
        _ => unimplemented!(),
    }
}

fn show_basic_agent_setting(row: &AgentQueryResult, agent_id: i32) -> (HeaderMap, Html<String>) {
    let name: String = row.name.clone();
    let description: String = if let Some(d) = row.description.clone() {
        d
    } else {
        "".into()
    };
    let _status = row.status.clone();
    let configuration = row.configuration.clone();
    let agent_setting =
        serde_json::from_value::<AgentSetting>(configuration.clone()).unwrap_or_default();

    let model_name = agent_setting.model_name;
    let fsm_agent_config = agent_setting.fsm_agent_config;

    let fsm_config = FSMAgentConfigBuilder::from_json(&fsm_agent_config)
        .unwrap_or_default()
        .build()
        .unwrap_or_default();
    let prompt = fsm_config.sys_prompt.clone();
    let follow_up_prompt = fsm_config
        .prompts
        .get("AskFollowUpQuestion")
        .unwrap_or(&"".to_string())
        .clone();
    println!(
        "agent: {}:{} // {} // {}",
        agent_id, name, description, configuration
    );

    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    let model_options = SUPPORTED_MODELS
        .iter()
        .map(|(this_model_name, _)| {
            if this_model_name == &model_name {
                format!(
                    r#" <option value="{}" selected>{}</option>"#,
                    this_model_name, this_model_name
                )
            } else {
                format!(
                    r#" <option value="{}">{}</option>"#,
                    this_model_name, this_model_name
                )
            }
        })
        .collect::<Vec<String>>();

    let out_html = if let Some(out_html) = {
        let template = UpdateBasicAgentTemplate {
            agent_id,
            name,
            description,
            model_options,
            prompt,
            follow_up_prompt,
        };
        Some(template.render().unwrap())
    } {
        out_html
    } else {
        format!(r#"<div id="workspace">agent_id: {agent_id}</div>"#)
    };

    (h, Html::from(out_html))
}

fn show_adv_agent_setting(row: &AgentQueryResult, agent_id: i32) -> (HeaderMap, Html<String>) {
    let name: String = row.name.clone();
    let description: String = if let Some(d) = row.description.clone() {
        d
    } else {
        "".into()
    };
    let _status = row.status.clone();
    let configuration = row.configuration.clone();
    let agent_setting =
        serde_json::from_value::<AgentSetting>(configuration.clone()).unwrap_or_default();

    let model_name = agent_setting.model_name;
    let fsm_agent_config = agent_setting.fsm_agent_config;

    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    let model_options = SUPPORTED_MODELS
        .iter()
        .map(|(this_model_name, _)| {
            if this_model_name == &model_name {
                format!(
                    r#" <option value="{}" selected>{}</option>"#,
                    this_model_name, this_model_name
                )
            } else {
                format!(
                    r#" <option value="{}">{}</option>"#,
                    this_model_name, this_model_name
                )
            }
        })
        .collect::<Vec<String>>();

    let out_html = if let Some(out_html) = {
        let template = UpdateAdvAgentTemplate {
            agent_id,
            name,
            description,
            model_options,
            agent_config: fsm_agent_config,
        };
        Some(template.render().unwrap())
    } {
        out_html
    } else {
        format!(r#"<div id="workspace">agent_id: {agent_id}</div>"#)
    };

    (h, Html::from(out_html))
}

async fn use_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(agent_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    println!("in use_agent: agent_id {}", agent_id);
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let name;
    let user_id;
    let user_data;
    let configuration;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard
            .get_user_data()
            .await
            .expect("database error! can't get user data");

        let db_pool = DB_POOL.clone();

        let row = sqlx::query!(
            "SELECT a.agent_id, a.name, a.description, a.status, a.configuration, a.user_id FROM agents a
JOIN users u ON a.user_id = u.user_id
WHERE u.username = $1 AND a.agent_id = $2;",
            user_data.username,
            agent_id
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        name = row.name;
        let description: String = if let Some(d) = row.description {
            d
        } else {
            "".into()
        };
        let _status = row.status;
        user_id = row.user_id;
        let model_name;
        configuration = if let Some(conf) = row.configuration {
            let model_setting: AgentSetting =
                serde_json::from_value::<AgentSetting>(conf.clone()).unwrap();
            model_name = model_setting.model_name;
            conf.to_string()
        } else {
            model_name = "".into();
            "".into()
        };

        println!(
            "agent: {}:{} // {} // {} // {}",
            agent_id, name, description, model_name, configuration
        );
    }
    {
        let ctx_guard = ctx.read().await;
        let mut assets_guard = ctx_guard.assets.write().await;
        assets_guard.insert("user_id".into(), TnAsset::U32(user_id as u32));
        assets_guard.insert("agent_name".into(), TnAsset::String(name.clone()));
        assets_guard.insert("agent_id".into(), TnAsset::U32(agent_id as u32));
        let uuid = Uuid::new_v4();
        let title = format!("{}:{}", name, uuid);
        let db_pool = DB_POOL.clone();
        let new_chat = sqlx::query!(
            r#"
            INSERT INTO chats (user_id, agent_id, title)
            SELECT u.user_id, $2, $3
            FROM users u
            WHERE u.username = $1
            RETURNING chat_id, created_at, updated_at
            "#,
            user_data.username,
            agent_id,
            title
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();
        let chat_id = new_chat.chat_id;
        assets_guard.insert("chat_id".into(), TnAsset::U32(chat_id as u32));
        assets_guard.insert("agent_configuration".into(), TnAsset::String(configuration));
    }
    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    {
        chatbox::clean_chatbox_with_context(ctx, AGENT_CHAT_TEXTAREA).await;
        text::clean_textarea_with_context(ctx, AGENT_QUERY_TEXT_INPUT).await;
    }
    update_and_send_textarea_with_context(&ctx, ASSET_SEARCH_OUTPUT, "").await;
    let out_html = {
        let ctx_guard = ctx.read().await;

        let component_guard = ctx_guard.components.read().await;
        let mut agent_ws = component_guard.get(AGENT_WORKSPACE).unwrap().write().await;
        agent_ws.pre_render(&ctx_guard).await;
        agent_ws.render().await
    };
    (h, Html::from(out_html))
}

use uuid::{timestamp::context, Uuid};

#[derive(Template)] // this will generate the code...
#[template(path = "simple_agent_config.json", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct SimpleAgentConfigTemplate {
    prompt: String,
    follow_up: String,
}

async fn create_basic_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!("payload: {}", payload);
    let _agent_configuration = payload.to_string();

    let agent_setting_form: SimpleAgentSettingForm =
        serde_json::from_value::<SimpleAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

        let prompt = serde_json::to_string_pretty(&agent_setting_form.prompt).unwrap(); 
        let follow_up = serde_json::to_string_pretty(&agent_setting_form.follow_up_prompt.unwrap_or("Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.".into())).unwrap(); 
        let simple_agent_config = SimpleAgentConfigTemplate {
            prompt,
            follow_up
        }.render().unwrap();

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        //provider: agent_setting_form.provider.clone(),
        fsm_agent_config: simple_agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    // TODO: make sure the string is proper avoiding SQL injection
    let _query = sqlx::query!(
        r#"INSERT INTO agents (user_id, name, description, status, configuration, class)
        SELECT user_id, $2, $3, $4, $5, $6
        FROM users
        WHERE username = $1"#,
        user_data.username,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        "basic".into()
    )
    .fetch_one(&db_pool)
    .await;

    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">An agent "{}" is created "#,
        agent_setting_form.name
    ))
}

async fn create_adv_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!("payload: {}", payload);
    let _agent_configuration = payload.to_string();

    let agent_setting_form: AdvAgentSettingForm =
        serde_json::from_value::<AdvAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        // provider: agent_setting_form.provider.clone(),
        fsm_agent_config: agent_setting_form.agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    // TODO: make sure the string is proper avoiding SQL injection
    let _query = sqlx::query!(
        r#"INSERT INTO agents (user_id, name, description, status, configuration, class)
        SELECT user_id, $2, $3, $4, $5, $6
        FROM users
        WHERE username = $1"#,
        user_data.username,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        "advanced".into()
    )
    .fetch_one(&db_pool)
    .await;

    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">An agent "{}" is created "#,
        agent_setting_form.name
    ))
}

async fn update_basic_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Path(agent_id): Path<i32>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!("in update_agent   payload: {}", payload);
    let _agent_configuration = payload.to_string();

    let agent_setting_form: SimpleAgentSettingForm =
        serde_json::from_value::<SimpleAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");
    let prompt = serde_json::to_string_pretty(&agent_setting_form.prompt).unwrap(); 
    let follow_up = serde_json::to_string_pretty(&agent_setting_form.follow_up_prompt.unwrap_or("Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.".into())).unwrap(); 
    tracing::info!(target: "tron_app", "prompt: {}", prompt);
    tracing::info!(target: "tron_app", "follow_up: {}", follow_up);
    let simple_agent_config = SimpleAgentConfigTemplate {
        prompt,
        follow_up
    }.render().unwrap();


    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        //provider: agent_setting_form.provider.clone(),
        fsm_agent_config: simple_agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    let _query = sqlx::query!(
        r#"UPDATE agents 
    SET name = $3, description = $4, status = $5, configuration = $6
    WHERE agent_id = $2 AND user_id = (SELECT user_id FROM users WHERE username = $1)"#,
        user_data.username,
        agent_id,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value
    )
    .fetch_one(&db_pool)
    .await;

    // println!("query rtn: {:?}", _query);
    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">The agent "{}" is updated "#,
        agent_setting_form.name
    ))
}

async fn update_adv_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Path(agent_id): Path<i32>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!("in update_agent   payload: {}", payload);
    let _agent_configuration = payload.to_string();

    let agent_setting_form: AdvAgentSettingForm =
        serde_json::from_value::<AdvAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

    // TODO: validate agent_config

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        // provider: agent_setting_form.provider.clone(),
        fsm_agent_config: agent_setting_form.agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    let _query = sqlx::query!(
        r#"UPDATE agents 
    SET name = $3, description = $4, status = $5, configuration = $6
    WHERE agent_id = $2 AND user_id = (SELECT user_id FROM users WHERE username = $1)"#,
        user_data.username,
        agent_id,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value
    )
    .fetch_one(&db_pool)
    .await;

    println!("query rtn: {:?}", _query);
    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">The agent "{}" is updated "#,
        agent_setting_form.name
    ))
}

async fn deactivate_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Path(agent_id): Path<i32>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!("in  deactivate_agent   payload: {}", payload);
    let _agent_configuration = payload.to_string();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");
    let db_pool = DB_POOL.clone();
    let row = sqlx::query!(
        r#"UPDATE agents SET status = $3
           WHERE agent_id = $2 AND user_id = (SELECT user_id FROM users WHERE username = $1)
           RETURNING name"#,
        user_data.username,
        agent_id,
        "inactive"
    )
    .fetch_one(&db_pool)
    .await
    .expect("sql query error");

    Html::from(format!(
        r#"<p class="py-4">The agent "{}" is deactivated"#,
        row.name
    ))
}

async fn check_user(_method: Method, State(appdata): State<Arc<AppData>>, session: Session) {
    // let user_data = session
    // .get::<String>("user_data")
    // .await
    // .expect("error on getting user data");

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard
        .get_user_data()
        .await
        .expect("database error! can't get user data");

    let db_pool = DB_POOL.clone();
    let res = sqlx::query!(
        r#"SELECT user_id FROM users WHERE username = $1"#,
        user_data.username
    )
    .fetch_one(&db_pool)
    .await;

    let user_id = if let Err(_res) = res {
        let rec = sqlx::query!(
            r#"INSERT INTO users (username, email, password_hash, created_at, last_login) VALUES
($1, $2, 'hashed_password', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
RETURNING user_id"#,
            user_data.username,
            user_data.email
        )
        .fetch_one(&db_pool)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "unable to active a new user, username {}",
                user_data.username
            )
        });
        rec.user_id
    } else {
        res.unwrap().user_id
    };
    println!("check_user: {:?} id: {}", user_data, user_id);
}

async fn show_chat(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    println!("in show_chat: chat_id {}", chat_id);
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let user_id;
    let agent_id;
    let agent_name;
    let user_data;
    let configuration;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard
            .get_user_data()
            .await
            .expect("database error! can't get user data");

        let db_pool = DB_POOL.clone();

        let row = sqlx::query!(
            "SELECT c.agent_id, c.user_id, a.name, a.configuration FROM chats c
JOIN users u ON c.user_id = u.user_id
JOIN agents a ON c.agent_id = a.agent_id
WHERE u.username = $1 AND c.chat_id = $2;",
            user_data.username,
            chat_id
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        user_id = row.user_id;
        agent_id = row.agent_id.unwrap_or_default();
        agent_name = row.name;
        configuration = if let Some(conf) = row.configuration {
            conf.to_string()
        } else {
            "".into()
        };
    }
    {
        let ctx_guard = ctx.read().await;
        let mut assets_guard = ctx_guard.assets.write().await;
        assets_guard.insert("user_id".into(), TnAsset::U32(user_id as u32));
        assets_guard.insert("agent_name".into(), TnAsset::String(agent_name.clone()));
        assets_guard.insert("agent_id".into(), TnAsset::U32(agent_id as u32));
        assets_guard.insert("chat_id".into(), TnAsset::U32(chat_id as u32));
        assets_guard.insert("agent_configuration".into(), TnAsset::String(configuration));
    }
    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    {
        //chatbox::clean_chatbox_with_context(ctx, AGENT_CHAT_TEXTAREA).await;
        text::clean_textarea_with_context(ctx, AGENT_QUERY_TEXT_INPUT).await;
    }
    let out_html = {
        let ctx_guard = ctx.read().await;

        let component_guard = ctx_guard.components.read().await;
        let mut agent_ws = component_guard.get(AGENT_WORKSPACE).unwrap().write().await;
        agent_ws.pre_render(&ctx_guard).await;
        agent_ws.render().await
    };
    (h, Html::from(out_html))
}

async fn delete_chat(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    println!("in delete_chat: chat_id {}", chat_id);
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    //let user_id: i32;
    //let agent_id: i32;
    //let agent_name: String;
    //let user_data;
    //let configuration: String;
    {
        let ctx_guard = ctx.read().await;
        // user_data = ctx_guard
        //     .get_user_data()
        //     .await
        //     .expect("database error! can't get user data");

        let db_pool = DB_POOL.clone();
        let row = sqlx::query!(
            r#"UPDATE chats SET status = $2
               WHERE chat_id = $1 RETURNING chat_id"#,
            chat_id,
            "inactive"
        )
        .fetch_one(&db_pool)
        .await
        .expect("sql query error");
    }
    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());
    let out_html = {
        let ctx_guard = ctx.read().await;

        let component_guard = ctx_guard.components.read().await;
        let mut agent_ws = component_guard.get(SESSION_CARDS).unwrap().write().await;
        agent_ws.pre_render(&ctx_guard).await;
        agent_ws.render().await
    };
    (h, Html::from(out_html))
}



async fn show_asset(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    println!("in show_asset: asset_id {}", chat_id);
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let user_data;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard
            .get_user_data()
            .await
            .expect("database error! can't get user data");
    }
    let mut h = HeaderMap::new();
    h.insert("Hx-Reswap", "outerHTML show:top".parse().unwrap());
    h.insert("Hx-Retarget", "#workspace".parse().unwrap());

    let out_html = {
        let ctx_guard = ctx.read().await;

        let component_guard = ctx_guard.components.read().await;
        let mut asset_space_plot = component_guard.get(ASSET_SPACE_PLOT).unwrap().write().await;
        asset_space_plot.pre_render(&ctx_guard).await;
        asset_space_plot.render().await
    };
    (h, Html::from(out_html))
}
