#![allow(dead_code)]
#![allow(unused_imports)]

mod agent_workspace;
mod asset_cards;
mod asset_space_plot;
mod embedding_service;
mod library_cards;
mod llm_agent;
mod services;
mod session_cards;

use agent_workspace::*;
use ai_gent_lib::llm_agent::{FSMAgentConfig, FSMAgentConfigBuilder};
use askama::Template;
use asset_cards::{AssetCards, AssetCardsBuilder};
use asset_space_plot::AssetSpacePlot;
use embedding_service::{DocumentChunk, DocumentChunks};
use futures_util::Future;
use library_cards::{LibraryCards, LibraryCardsBuilder};
use pgvector::Vector;
use session_cards::{SessionCards, SessionCardsBuilder};

use axum::{
    body::Body, extract::{Json, Path, State}, http::{header, HeaderMap, Method, StatusCode}, response::{Html, IntoResponse, Redirect}, routing::{get, post, trace}, Router
};
use serde::{Deserialize, Serialize};
//use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::Sender, Mutex, RwLock};

use serde_json::{json, Value};

use tower_sessions::{cookie::time::Time, Session};
use tracing::debug;
use tron_app::{
    tron_components::{
        self,
        button::TnButtonBuilder,
        chatbox,
        text::{self, update_and_send_textarea_with_context},
        tn_future, TnActionExecutionMethod, TnAsset, TnComponentBaseRenderTrait,
        TnComponentBaseTrait, TnDnDFileUpload, TnFutureHTMLResponse, TnFutureString,
        TnHtmlResponse, TnServiceRequestMsg, UserData,
    },
    AppData, HtmlAttributes, Ports, TRON_APP,
};
use tron_components::{
    text::TnTextInput, TnButton, TnComponentState, TnComponentValue, TnContext, TnContextBase,
    TnEvent, TnTextArea,
};
//use std::sync::Mutex;
use sqlx::{any::AnyRow, prelude::FromRow, query::Query, Acquire};
use sqlx::{postgres::types::PgRange, Postgres};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::{
    collections::HashMap, default, fs::File, ops::Bound, pin::Pin, sync::Arc, task::Context,
};

use html_escape::encode_text;
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
    model_name: String,
    asset_id: String,
    prompt: String,
    follow_up_prompt: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AdvAgentSettingForm {
    name: String,
    description: String,
    model_name: String,
    asset_id: String,
    agent_config: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AgentSetting {
    name: String,
    description: String,
    model_name: String,
    fsm_agent_config: String,
}

static BUTTON: &str = "button";
static LIBRARY_CARDS: &str = "lib_cards";
static SESSION_CARDS: &str = "session_cards";
static ASSET_CARDS: &str = "asset_cards";
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
    ("o3-mini", "OPENAI_API_KEY"),
    ("claude-3-haiku-20240307", "ANTHROPIC_API_KEY"),
    ("claude-3-5-sonnet-20241022", "ANTHROPIC_API_KEY"),
];

static MOCK_USER: Lazy<UserData> = Lazy::new(|| UserData {
    username: "user".into(),
    email: "user@test.com".into(),
});


use time::Duration;

// This is the main entry point of the application
// It sets up the application configuration and state
// and then starts the application by calling tron_app::run
#[tokio::main]
async fn main() {
    embedding_service::initialize_embedding_model().await;
    let ui_action_routes = Router::<Arc<AppData>>::new()
        .route("/service/session-check", get(session_check))
        .route("/agent/create", post(create_basic_agent))
        .route("/agent/create_adv", post(create_adv_agent))
        .route("/agent/{id}/update", post(update_basic_agent))
        .route("/agent/{id}/update_adv", post(update_adv_agent))
        .route("/agent/{id}/use", get(use_agent))
        .route("/agent/{id}/show", get(show_agent_setting))
        .route("/agent/{id}/deactivate", post(deactivate_agent))
        .route("/chat/{id}/delete", get(delete_chat))
        .route("/chat/{id}/show", get(show_chat))
        .route("/chat/{id}/download", get(download_chat))
        .route("/asset/{id}/show", get(show_asset))
        .route("/asset/create", post(create_asset))
        .route("/asset/{id}/delete", get(delete_asset))
        .route("/check_user", get(check_user))
        .route(
            "/service/text_to_embedding",
            post(services::text_to_embedding),
        );

    let app_config = tron_app::AppConfigure {
        cognito_login: false,
        http_only: true,
        address: [0, 0, 0, 0],
        ports: Ports {
            https: 3001,
            http: 8080,
        },
        api_router: Some(ui_action_routes),
        session_expiry: Some(Duration::minutes(10)),
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

    AssetCards::builder()
        .init(ASSET_CARDS.into(), "cards".into(), "active")
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

const SHOW_ASSET_LIB_BTN: &str = "show_asset_btn";
const CREATE_ASSET_LIB_BTN: &str = "create_asset_btn";

static DND_FILE_UPLOAD: &str = "dnd_file_upload";

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
        .init(SHOW_TODAY_SESSION_BTN.into(), "Last 24 hours".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_YESTERDAY_SESSION_BTN.into(), "Last 48 hours".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_LAST_WEEK_SESSION_BTN.into(), "Last Week".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_ALL_SESSION_BTN.into(), "All".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(SHOW_ASSET_LIB_BTN.into(), "Show Assets".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    TnButton::builder()
        .init(CREATE_ASSET_LIB_BTN.into(), "Create Assets".into())
        .update_attrs(attrs.clone())
        .set_action(TnActionExecutionMethod::Await, change_workspace)
        .add_to_context(ctx);

    add_dnd_file_upload(ctx, DND_FILE_UPLOAD);
}

fn add_dnd_file_upload(context: &mut TnContextBase, tnid: &str) {
    let button_attributes = vec![(
        "class".into(),
        "btn btn-sm btn-outline btn-primary flex-1".into(),
    )]
    .into_iter()
    .collect::<HashMap<String, String>>();

    TnDnDFileUpload::builder()
        .init(
            tnid.into(),
            "Drop An Asset JSON File".into(),
            button_attributes,
        )
        .set_action(TnActionExecutionMethod::Await, handle_file_upload)
        .add_to_context(context);
}

#[derive(Template)] // this will generate the code...
#[template(path = "app_page.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AppPageTemplate {
    library_cards: String,
    agent_buttons: Vec<String>,
    sessions_buttons: Vec<String>,
    assets_buttons: Vec<String>,
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
        let mut assets_buttons = Vec::<String>::new();
        for btn in [SHOW_ASSET_LIB_BTN, CREATE_ASSET_LIB_BTN] {
            assets_buttons.push(context_guard.get_rendered_string(btn).await);
        }
        let html = AppPageTemplate { library_cards, agent_buttons, sessions_buttons, assets_buttons };
        html.render().unwrap()
    }
}

#[derive(Template)]
#[template(path = "create_basic_agent.html", escape = "none")]
struct SetupAgentTemplate {
    model_options: Vec<String>,
    asset_options: Vec<String>,
}

#[derive(Template)]
#[template(path = "update_basic_agent.html", escape = "none")]
struct UpdateBasicAgentTemplate {
    agent_id: i32,
    name: String,
    description: String,
    model_options: Vec<String>,
    asset_options: Vec<String>,
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
    asset_options: Vec<String>,
    agent_config: String,
}

#[derive(Template)]
#[template(path = "create_fsm_agent.html", escape = "none")]
struct SetupAdvAgentTemplate {
    model_options: Vec<String>,
    asset_options: Vec<String>,
}

#[derive(Template)]
#[template(path = "user_settings.html")]
struct UserSettingsTemplate {
    username: String,
    email: String,
    anthropic_api_key: String,
    openai_api_key: String,
}

#[derive(Template)]
#[template(path = "create_asset.html", escape = "none")]
struct CreateAgentTemplate {
    dnd_file_upload_html: String,
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

                let ctx_guard = context.read().await;
                let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
                let asset_list = get_active_asset_list(&user_data.username).await;
                let mut asset_options = vec![r#"<option value=0 selected>No Asset</option>"#.to_string()];
                asset_options.extend(asset_list.into_iter().map(|(id, name)| {
                    format!(r#" <option value={}>{}</option>"#, id, encode_text(&name)) } ));
                let template = SetupAgentTemplate { model_options, asset_options };
                Some(template.render().unwrap())
            },

            ADV_AGENT_DESIGN_BTN => {
                let model_options = SUPPORTED_MODELS.iter().map( | (model_name, _) |
                    format!(r#" <option value="{}">{}</option>"#, model_name, model_name) ).collect::<Vec<String>>();

                let ctx_guard = context.read().await;
                let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
                let asset_list = get_active_asset_list(&user_data.username).await;
                let mut asset_options = vec![r#"<option value=0 selected>No Asset</option>"#.to_string()];
                asset_options.extend(asset_list.into_iter().map(|(id, name)| {
                    format!(r#" <option value={}>{}</option>"#, id, encode_text(&name)) } ));

                let template = SetupAdvAgentTemplate { model_options, asset_options};
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

            SHOW_ASSET_LIB_BTN => {
                let context_guard = context.read().await;
                let cards = context_guard.get_initial_rendered_string(ASSET_CARDS).await;
                Some(cards)
            },

            CREATE_ASSET_LIB_BTN => {

                // clear the upload buffer
                {
                    let asset_ref = context.get_asset_ref().await;
                    let mut guard = asset_ref.write().await;
                    if let Some(TnAsset::HashMapVecU8(h)) = guard.get_mut("upload") {
                            h.clear();
                    }
                }

                {
                    let context_guard = context.read().await;
                    let dnd_file_upload_html = context_guard.get_rendered_string(DND_FILE_UPLOAD).await;
                    let template = CreateAgentTemplate {dnd_file_upload_html};
                    Some(template.render().unwrap())
                }
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
    asset_id: Option<i32>,
}

async fn show_agent_setting(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(agent_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    // tracing::info!(target: "tron_app", "in show_agent: agent_id {}", agent_id);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

    let db_pool = DB_POOL.clone();

    let row = sqlx::query_as!(
        AgentQueryResult,
        "SELECT a.agent_id, a.name, a.description, a.status, a.configuration, a.class, a.asset_id FROM agents a
JOIN users u ON a.user_id = u.user_id
WHERE u.username = $1 AND a.agent_id = $2;",
        user_data.username,
        agent_id,
    )
    .fetch_one(&db_pool)
    .await
    .unwrap();

    let asset_id = row.asset_id.unwrap_or(0);
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
    let mut asset_list = vec![(0_i32, "No Asset".to_string())];
    asset_list.extend(get_active_asset_list(&user_data.username).await);
    let asset_options = asset_list
        .into_iter()
        .map(|(id, name)| {
            if id == asset_id {
                format!(
                    r#" <option value={} selected>{}</option>"#,
                    id,
                    encode_text(&name)
                )
            } else {
                format!(r#" <option value={}>{}</option>"#, id, encode_text(&name))
            }
        })
        .collect();

    match row.class.as_str() {
        "basic" => show_basic_agent_setting(&row, asset_options, agent_id),
        "advanced" => show_adv_agent_setting(&row, asset_options, agent_id),
        _ => unimplemented!(),
    }
}

fn show_basic_agent_setting(
    row: &AgentQueryResult,
    asset_options: Vec<String>,
    agent_id: i32,
) -> (HeaderMap, Html<String>) {
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
    // tracing::info!(
    //     target: "tron_app",
    //     "agent: {}:{} // {} // {}",
    //     agent_id, name, description, configuration
    // );

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
            asset_options,
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

fn show_adv_agent_setting(
    row: &AgentQueryResult,
    asset_options: Vec<String>,
    agent_id: i32,
) -> (HeaderMap, Html<String>) {
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
            asset_options,
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
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let name;
    let user_id;
    let user_data;
    let asset_id;
    let configuration;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

        let db_pool = DB_POOL.clone();

        let row = sqlx::query!(
            "SELECT a.agent_id, a.name, a.description, a.status, a.configuration, a.user_id, a.asset_id, assets.status asset_status FROM agents a
             JOIN users u ON a.user_id = u.user_id
             JOIN assets ON assets.asset_id = a.asset_id
             WHERE u.username = $1 AND a.agent_id = $2;",
            user_data.username,
            agent_id
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        name = row.name;
        let _description: String = if let Some(d) = row.description {
            d
        } else {
            "".into()
        };
        let _status = row.status;
        user_id = row.user_id;

        let _model_name;
        configuration = if let Some(conf) = row.configuration {
            let model_setting: AgentSetting =
                serde_json::from_value::<AgentSetting>(conf.clone()).unwrap();
            _model_name = model_setting.model_name;
            conf.to_string()
        } else {
            _model_name = "".into();
            "".into()
        };

        // TODO: check if the asset is still active
        asset_id = if let Some(asset_id) = row.asset_id {
            if row.asset_status == "active" {
                asset_id as u32
            } else {
                0_u32
            }
        } else {
            0_u32
        };

        // tracing::info!(
        //     target: "tron_app",
        //     "agent: {}:{} // {} // {} // {} // {:?}",
        //     agent_id, name, description, model_name, configuration, asset_id
        // );
    }
    {
        let ctx_guard = ctx.read().await;
        let mut assets_guard = ctx_guard.assets.write().await;
        assets_guard.insert("user_id".into(), TnAsset::U32(user_id as u32));
        assets_guard.insert("agent_name".into(), TnAsset::String(name.clone()));
        assets_guard.insert("agent_id".into(), TnAsset::U32(agent_id as u32));
        assets_guard.insert("asset_id".into(), TnAsset::U32(asset_id));
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
    update_and_send_textarea_with_context(ctx, ASSET_SEARCH_OUTPUT, "").await;
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
#[template(path = "simple_agent_config.json.template", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
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
    let _agent_configuration = payload.to_string();

    let agent_setting_form: SimpleAgentSettingForm =
        serde_json::from_value::<SimpleAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

    let prompt = serde_json::to_string_pretty(&agent_setting_form.prompt).unwrap();
    let follow_up = serde_json::to_string_pretty(&agent_setting_form.follow_up_prompt.unwrap_or("Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.".into())).unwrap();
    let simple_agent_config = SimpleAgentConfigTemplate { prompt, follow_up }
        .render()
        .unwrap();

    let asset_id = agent_setting_form.asset_id.parse::<i32>();
    let asset_id = if let Ok(asset_id) = asset_id {
        if asset_id != 0 {
            Some(asset_id)
        } else {
            None
        }
    } else {
        None
    };

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        fsm_agent_config: simple_agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    // TODO: make sure the string is proper avoiding SQL injection
    let _query = sqlx::query!(
        r#"INSERT INTO agents (user_id, name, description, status, configuration, class, asset_id)
        SELECT user_id, $2, $3, $4, $5, $6, $7
        FROM users
        WHERE username = $1"#,
        user_data.username,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        "basic".into(),
        asset_id
    )
    .fetch_one(&db_pool)
    .await;

    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">An agent "{}" is created </p>"#,
        agent_setting_form.name
    ))
}

async fn create_adv_agent(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let _agent_configuration = payload.to_string();

    let agent_setting_form: AdvAgentSettingForm =
        serde_json::from_value::<AdvAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

    let asset_id = agent_setting_form.asset_id.parse::<i32>();
    let asset_id = if let Ok(asset_id) = asset_id {
        if asset_id != 0 {
            Some(asset_id)
        } else {
            None
        }
    } else {
        None
    };

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        fsm_agent_config: agent_setting_form.agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    // TODO: make sure the string is proper avoiding SQL injection
    let _query = sqlx::query!(
        r#"INSERT INTO agents (user_id, name, description, status, configuration, class, asset_id)
        SELECT user_id, $2, $3, $4, $5, $6, $7
        FROM users
        WHERE username = $1"#,
        user_data.username,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        "advanced".into(),
        asset_id
    )
    .fetch_one(&db_pool)
    .await;

    //let uuid = Uuid::new_v4();
    Html::from(format!(
        r#"<p class="py-4">An agent "{}" is created </p>"#,
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
    let _agent_configuration = payload.to_string();

    let agent_setting_form: SimpleAgentSettingForm =
        serde_json::from_value::<SimpleAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
    let prompt = serde_json::to_string_pretty(&agent_setting_form.prompt).unwrap();
    let follow_up = serde_json::to_string_pretty(&agent_setting_form.follow_up_prompt.unwrap_or("Your goal to see if you have enough information to address the user's question, if not, please ask more questions for the information you need.".into())).unwrap();
    // tracing::info!(target: "tron_app", "prompt: {}", prompt);
    // tracing::info!(target: "tron_app", "follow_up: {}", follow_up);
    let simple_agent_config = SimpleAgentConfigTemplate { prompt, follow_up }
        .render()
        .unwrap();

    let asset_id = agent_setting_form.asset_id.parse::<i32>();
    let asset_id = if let Ok(asset_id) = asset_id {
        if asset_id != 0 {
            Some(asset_id)
        } else {
            None
        }
    } else {
        None
    };
    //tracing::info!(target: TRON_APP, "update basis agent asset id: {:?} {:?}", agent_setting_form.asset_id, asset_id);
    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        fsm_agent_config: simple_agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    let _query = sqlx::query!(
        r#"UPDATE agents 
    SET name = $3, description = $4, status = $5, configuration = $6, asset_id = $7
    WHERE agent_id = $2 AND user_id = (SELECT user_id FROM users WHERE username = $1)"#,
        user_data.username,
        agent_id,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        asset_id
    )
    .fetch_one(&db_pool)
    .await;

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
    let _agent_configuration = payload.to_string();

    let agent_setting_form: AdvAgentSettingForm =
        serde_json::from_value::<AdvAgentSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

    let asset_id = agent_setting_form.asset_id.parse::<i32>();
    let asset_id = if let Ok(asset_id) = asset_id {
        if asset_id != 0 {
            Some(asset_id)
        } else {
            None
        }
    } else {
        None
    };

    // TODO: validate agent_config

    let agent_setting = AgentSetting {
        name: agent_setting_form.name.clone(),
        model_name: agent_setting_form.model_name.clone(),
        description: agent_setting_form.description.clone(),
        fsm_agent_config: agent_setting_form.agent_config,
    };

    let agent_setting_value = serde_json::to_value(&agent_setting).unwrap();

    let db_pool = DB_POOL.clone();
    let _query = sqlx::query!(
        r#"UPDATE agents 
    SET name = $3, description = $4, status = $5, configuration = $6, asset_id = $7
    WHERE agent_id = $2 AND user_id = (SELECT user_id FROM users WHERE username = $1)"#,
        user_data.username,
        agent_id,
        agent_setting_form.name,
        agent_setting_form.description,
        "active",
        agent_setting_value,
        asset_id
    )
    .fetch_one(&db_pool)
    .await;

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
    let _agent_configuration = payload.to_string();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let ctx_guard = ctx.read().await;
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
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
    let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

    let db_pool = DB_POOL.clone();
    let res = sqlx::query!(
        r#"SELECT user_id FROM users WHERE username = $1"#,
        user_data.username
    )
    .fetch_one(&db_pool)
    .await;

    let _user_id = if let Err(_res) = res {
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
    //tracing::info!(target: "tron_app", "check_user: {:?} id: {}", user_data, user_id);
}

async fn show_chat(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let user_id;
    let agent_id;
    let asset_id;
    let agent_name;
    let user_data;
    let configuration;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

        let db_pool = DB_POOL.clone();

        let row = sqlx::query!(
            "SELECT c.agent_id, c.user_id, a.name, a.configuration, a.asset_id FROM chats c
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
        asset_id = if let Some(asset_id) = row.asset_id {
            asset_id as u32
        } else {
            0_u32
        };
    }
    {
        let ctx_guard = ctx.read().await;
        let mut assets_guard = ctx_guard.assets.write().await;
        assets_guard.insert("user_id".into(), TnAsset::U32(user_id as u32));
        assets_guard.insert("agent_name".into(), TnAsset::String(agent_name.clone()));
        assets_guard.insert("agent_id".into(), TnAsset::U32(agent_id as u32));
        assets_guard.insert("chat_id".into(), TnAsset::U32(chat_id as u32));
        assets_guard.insert("asset_id".into(), TnAsset::U32(asset_id));
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

#[derive(Serialize)]
struct SingleChatMessage {
    time_stamp: String,
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatDownload {
    messages: Vec<SingleChatMessage>,
    summary: String,
}

async fn download_chat(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    let user_data;
    {
        let ctx_guard = ctx.read().await;
        user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());
    }
    let pool = DB_POOL.clone();
    let results = sqlx::query!(
        r#"
        SELECT m.timestamp, m.role, m.content
        FROM messages m 
        JOIN chats c ON c.chat_id = m.chat_id 
        JOIN users u ON c.user_id = u.user_id
        WHERE m.chat_id = $1 AND c.status = $2 AND u.username = $3
        ORDER BY m.timestamp ASC
        "#,
        chat_id,
        "active",
        user_data.username
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let messages = results
        .into_iter()
        .map(|row| SingleChatMessage {
            time_stamp: row.timestamp.unwrap_or_default().to_string(),
            role: row.role.unwrap_or_default(),
            content: row.content,
        })
        .collect::<Vec<_>>();

    let result = sqlx::query!(
        r#" 
        SELECT summary FROM chats c
        JOIN users u on c.user_id = u.user_id 
        WHERE c.chat_id = $1 AND c.status = $2 AND u.username = $3"#,
        chat_id,
        "active",
        user_data.username
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let summary = result.summary.unwrap_or_default();
    let chat_download = ChatDownload { messages, summary };
    let chat_download = serde_json::to_string_pretty(&chat_download).unwrap();

    axum::response::Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(chat_download)
        .unwrap()
}

async fn delete_chat(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(chat_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    {
        let db_pool = DB_POOL.clone();
        let _row = sqlx::query!(
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
    Path(asset_id): Path<u32>,
    session: Session,
) -> impl IntoResponse {
    //println!("payload: {:?}", payload);
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();

    {
        let ctx_guard = ctx.read().await;
        let mut assets_guard = ctx_guard.assets.write().await;
        assets_guard.insert("asset_id".into(), TnAsset::U32(asset_id));
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

async fn delete_asset(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    Path(asset_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();
    {
        let db_pool = DB_POOL.clone();
        let _row = sqlx::query!(
            r#"UPDATE assets SET status = $2
               WHERE asset_id = $1 RETURNING asset_id"#,
            asset_id,
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
        let mut agent_ws = component_guard.get(ASSET_CARDS).unwrap().write().await;
        agent_ws.pre_render(&ctx_guard).await;
        agent_ws.render().await
    };
    (h, Html::from(out_html))
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AssetSettingForm {
    name: String,
    description: String,
}

async fn create_asset(
    _method: Method,
    State(appdata): State<Arc<AppData>>,
    session: Session,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    // tracing::info!(target: "app_tron", "payload create_asset {:?}", payload);

    let asset_setting_form: AssetSettingForm =
        serde_json::from_value::<AssetSettingForm>(payload.clone()).unwrap();

    let ctx_store_guard = appdata.context_store.read().await;
    let ctx = ctx_store_guard.get(&session.id().unwrap()).unwrap();

    let document_chunks = {
        let asset_ref = ctx.get_asset_ref().await;
        let mut chunks = DocumentChunks { chunks: vec![] };
        let guard = asset_ref.read().await;
        if let Some(TnAsset::VecString2(asset_files)) = guard.get("asset_files") {
            for (filename, t) in asset_files {
                tracing::info!(target: TRON_APP, "process upload files {} {}", filename, t);
                let asset = guard.get("upload").unwrap();
                let file_data = if let TnAsset::HashMapVecU8(h) = asset {
                    h.get(filename)
                } else {
                    None
                };
                if let Some(data) = file_data {
                    let c = match t.as_str() {
                        "application/x-gzip" => DocumentChunks::from_gz_data(data),
                        "application/json" | "" => DocumentChunks::from_data(data), // we may use .jsonl than just .json
                        _ => None,
                    };
                    if let Some(c) = c {
                        chunks.chunks.extend(c.chunks);
                    }
                }
            }
        };
        chunks
    };

    tracing::info!(target: TRON_APP, "number of chunks parsed: {}", document_chunks.chunks.len());
    // clear the upload buffer
    {
        let asset_ref = ctx.get_asset_ref().await;
        let mut guard = asset_ref.write().await;
        if let Some(TnAsset::HashMapVecU8(h)) = guard.get_mut("upload") {
            h.clear();
        }
    }

    if !document_chunks.chunks.is_empty() {
        let ctx_guard = ctx.read().await;
        let user_data = ctx_guard.get_user_data().await.unwrap_or(MOCK_USER.clone());

        let db_pool = DB_POOL.clone();
        // TODO: make sure the string is proper avoiding SQL injection
        let query_result = sqlx::query!(
            r#"INSERT INTO assets (user_id, name, description, status)
        SELECT user_id, $2, $3, $4
        FROM users
        WHERE username = $1
        RETURNING asset_id"#,
            user_data.username,
            asset_setting_form.name,
            asset_setting_form.description,
            "active",
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        for c in document_chunks.chunks.into_iter() {
            let span = PgRange {
                start: Bound::Included(c.span.0 as i32),
                end: Bound::Excluded(c.span.1 as i32),
            };
            let embedding_vector = if let Some(v) = c.embedding_vec {
                Vector::from(v)
            } else {
                Vector::from(vec![])
            };
            let two_d_embedding = if let Some(v) = c.two_d_embedding {
                Vector::from(vec![v.0, v.1])
            } else {
                Vector::from(vec![])
            };
            let _res = sqlx::query(
             r#"INSERT INTO text_embedding (asset_id, text, span, embedding_vector, two_d_embedding, filename, title)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id"#   
            ).bind(query_result.asset_id)
            .bind(&c.text)
            .bind(span)
            .bind(embedding_vector)
            .bind(two_d_embedding)
            .bind(&c.filename)
            .bind(&c.title).fetch_one(&db_pool).await;
            // tracing::info!(target: TRON_APP, "insert embedding {:?}", _res);
        }

        //let uuid = Uuid::new_v4();
        Html::from(format!(
            r#"<p class="py-4">An Asset Collection "{}" is created </p>"#,
            asset_setting_form.name
        ))
    } else {
        Html::from(r#"<p class="py-4">No Valid Asset Data Uploaded</p>"#.to_string())
    }
}

fn handle_file_upload(context: TnContext, _event: TnEvent, payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        // process the "finished" event

        tracing::info!(target: TRON_APP, "process file_upload finish");
        let file_list = payload["event_data"]["e_file_list"].as_array();

        let file_list = if let Some(file_list) = file_list {
            file_list
                .iter()
                .flat_map(|v| {
                    if let Value::Array(v) = v {
                        let filename = v[0].as_str();
                        let size = v[1].as_u64();
                        let t = v[2].as_str();
                        match (filename, size, t) {
                            (Some(filename), Some(size), Some(t)) => Some((filename, size, t)),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        tracing::info!( target: TRON_APP, "uploaded files: {:?}", file_list);

        if !file_list.is_empty() {
            let mut v: Vec<(String, String)> = vec![];
            for (filename, _size, t) in file_list {
                    v.push( (filename.to_string(), t.to_string()) );
                }
            let asset_ref = context.get_asset_ref().await;
            let mut guard = asset_ref.write().await;
            guard.insert("asset_files".to_string(), TnAsset::VecString2(v));
        }

        let header = HeaderMap::new();
        Some((header, Html::from("".to_string())))
    }
}

async fn get_active_asset_list(username: &str) -> Vec<(i32, String)> {
    let pool = DB_POOL.clone();
    let query = format!(
        "SELECT a.asset_id, a.name, a.description
        FROM assets a
        JOIN users u ON a.user_id = u.user_id
        WHERE u.username = '{}' AND a.status = '{}' ORDER BY a.asset_id ASC;",
        username, "active"
    );

    let rows = sqlx::query(&query)
        .fetch_all(&pool)
        .await
        .expect("db error");

    let rtn = rows
        .iter()
        .map(|row| {
            let id: i32 = row.try_get("asset_id").unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            (id, name)
        })
        .collect::<Vec<_>>();
    rtn
}


async fn session_check(
    session: Session,
) -> impl IntoResponse {

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());

    if session.id().is_some() {
        (StatusCode::OK, response_headers, Body::default())
    } else {
        (StatusCode::UNAUTHORIZED, response_headers, Body::default())
    }
}