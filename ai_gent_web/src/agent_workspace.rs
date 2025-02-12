use ai_gent_lib::fsm::FSMBuilder;
use ai_gent_lib::fsm::FSMState;
use ai_gent_lib::fsm::FSMStateInit;
use ai_gent_lib::fsm::FSM;
use ai_gent_lib::llm_agent;
use ai_gent_lib::llm_agent::AgentSettings;
use ai_gent_lib::llm_agent::FSMAgentConfig;
use ai_gent_lib::llm_agent::FSMAgentConfigBuilder;
use ai_gent_lib::llm_agent::LLMAgent;
use ai_gent_lib::llm_agent::LLMClient;
use ai_gent_lib::GenaiLlmclient;
use askama::Template;
use async_trait::async_trait;
use axum::http::header;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::IntoResponse;
use candle_core::WithDType;
use futures::StreamExt;
use genai::chat::ChatMessage;
use ordered_float::OrderedFloat;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::default;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tron_app::tron_components::div::clean_div_with_context;
use tron_app::tron_components::div::update_and_send_div_with_context;
use tron_app::tron_components::text::append_textarea_value;
use tron_app::tron_components::text::clean_textarea_with_context;
// use tron_app::tron_components::text::update_and_send_textarea_with_context;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;
use tron_app::TRON_APP;

use super::DB_POOL;
use crate::embedding_service::vector_query_and_sort_points;
use crate::embedding_service::TextChunkingService;
use crate::embedding_service::TwoDPoint;
use crate::AgentSetting;
use crate::SEARCH_AGENT_BTN;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::Acquire;
use sqlx::Postgres;
use sqlx::{any::AnyRow, prelude::FromRow, query as sqlx_query};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use crate::embedding_service::EMBEDDING_SERVICE;

pub const AGENT_CHAT_TEXTAREA: &str = "agent_chat_textarea";
pub const AGENT_STREAM_OUTPUT: &str = "agent_stream_output";
pub const AGENT_QUERY_TEXT_INPUT: &str = "agent_query_text_input";
pub const AGENT_QUERY_BUTTON: &str = "agent_query_button";
pub const ASSET_SEARCH_BUTTON: &str = "asset_search_button";
pub const ASSET_SEARCH_OUTPUT: &str = "asset_search_output";
pub const TOPK_SLIDER: &str = "topk_slider";
pub const THRESHOLD_SLIDER: &str = "threshold_slider";
pub const TEMPERATURE_SLIDER: &str = "temperature_slider";
//pub const AGENT_NEW_SESSION_BUTTON: &str = "agent_new_session_button";

#[derive(Debug)]
pub struct FSMChatState {
    name: String,
    attributes: HashMap<String, String>,
    handle: Option<JoinHandle<String>>,
}

impl FSMStateInit for FSMChatState {
    fn new(name: &str, prompt: &str) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("prompt".to_string(), prompt.to_string());
        FSMChatState {
            name: name.to_string(),
            attributes,
            handle: None,
        }
    }
}

#[async_trait]
impl FSMState for FSMChatState {
    async fn on_enter(&self) {
        tracing::info!(target: TRON_APP, "Entering state: {}", self.name);
    }

    async fn on_exit(&self) {
        tracing::info!(target: TRON_APP, "Exiting state: {}", self.name);
    }

    async fn on_enter_mut(&mut self) {
        tracing::info!(target: TRON_APP, "Entering state (mut): {}", self.name);
    }

    async fn on_exit_mut(&mut self) {
        tracing::info!(target: TRON_APP, "Exiting state (mut): {}", self.name);
    }

    async fn serve(
        &mut self,
        tx: Sender<(String, String)>,
        _rx: Option<Receiver<(String, String)>>,
    ) {
        let llm_req_setting: llm_agent::LLMReqSetting =
            serde_json::from_str(&self.get_attribute("llm_req_setting").await.unwrap()).unwrap();
        let prompt = self.get_attribute("prompt").await;

        let full_prompt = match prompt {
            Some(prompt) => match llm_req_setting.context {
                Some(context) => [
                    &llm_req_setting.sys_prompt,
                    prompt.as_str(),
                    "\nHere is the summary of previous chat:\n",
                    "<SUMMARY>",
                    &llm_req_setting.summary,
                    "</SUMMARY>",
                    "\nHere is the current reference context:\n",
                    "<REFERENCES>",
                    &context,
                    "</REFERENCES>",
                ]
                .join("\n"),
                None => [
                    &llm_req_setting.sys_prompt,
                    prompt.as_str(),
                    "\nHere is the summary of previous chat:\n",
                    "<SUMMARY>",
                    &llm_req_setting.summary,
                    "</SUMMARY>",
                    "\nHere is the current reference context:\n",
                ]
                .join("\n"),
            },
            None => "".into(),
        };
        if full_prompt.is_empty() {
            let _ = tx.send(("error".into(), "no state prompt".into())).await;
            return;
        };
        let llm_client = GenaiLlmclient {
            model: llm_req_setting.model,
            api_key: llm_req_setting.api_key,
        };
        let messages = llm_req_setting.messages;
        let temperature = llm_req_setting.temperature;
        self.handle = Some(tokio::spawn(async move {
            let _ = tx
                .send((
                    "message".into(),
                    "LLM request sent, waiting for response\n".into(),
                ))
                .await;
            let mut llm_output = String::default();
            let mut llm_stream = llm_client
                .generate_stream(&full_prompt, &messages, temperature)
                .await;
            while let Some(result) = llm_stream.next().await {
                if let Some(output) = result {
                    llm_output.push_str(&output);
                    let _ = tx.send(("token".into(), output)).await;
                };
            }
            let _ = tx.send(("llm_output".into(), llm_output.clone())).await;
            llm_output
        }));

        if let Some(handle) = self.handle.take() {
            let llm_output = tokio::join!(handle);
            let llm_output = llm_output.0.unwrap();
            self.set_attribute("llm_output", llm_output).await;
        } else {
            self.set_attribute("llm_output", "".into()).await;
        }
    }

    async fn set_attribute(&mut self, k: &str, v: String) {
        self.attributes.insert(k.to_string(), v);
    }

    async fn get_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }

    async fn clone_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[non_exhaustive]
#[derive(ComponentBase)]
pub struct AgentWorkSpace<'a: 'static> {
    base: TnComponentBase<'a>,
    html: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "agent_workspace.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AgentWorkspaceTemplate {
    agent_name: String,
    agent_id: u32,
    chat_id: i32,
    asset_id: i32,
    chat_textarea: String,
    query_text_input: String,
    stream_output: String,
    query_button: String,
    asset_search_button: String,
    asset_search_output: String,
    topk_slider_html: String,
    threshold_slider_html: String,
    temperature_slider_html: String,
}

impl<'a: 'static> AgentWorkSpaceBuilder<'a> {
    pub fn init(mut self, tnid: String, title: String, context: &mut TnContextBase) -> Self {
        let component_type = TnComponentType::UserDefined("div".into());

        self.base = TnComponentBase::builder(self.base)
            .init("div".into(), tnid, component_type)
            .set_value(TnComponentValue::String(title))
            .build();

        let chat_textarea = TnChatBox::builder()
            .init(AGENT_CHAT_TEXTAREA.into(), vec![])
            .set_attr(
                "class",
                "min-h-[435px] max-h-[435px] overflow-auto  flex-1 p-2 border-2 mb-1 border-gray-600 rounded-lg p-1 h-min bg-gray-400",
            )
            .build();

        let mut query_text_input = TnTextArea::builder()
            .init(AGENT_QUERY_TEXT_INPUT.into(), "".into())
            .set_attr(
                "class",
                "flex-1 w-5/6 p-1 border-2 mb-1 border-gray-600 rounded-lg bg-gray-400 text-black",
            )
            .set_attr("style", "resize:none")
            .set_attr("hx-trigger", "change, server_event")
            .set_attr("hx-vals", r##"js:{event_data:get_input_event(event)}"##)
            .build(); //over-ride the default as we need the value of the input text
        query_text_input.remove_attribute("disabled");

        let agent_stream_output = TnStreamTextArea::builder()
            .init(AGENT_STREAM_OUTPUT.into(), Vec::new())
            .set_attr(
                "class",
                "min-h-[55px] flex-1 border-2 mb-1 border-gray-600 rounded-lg p-1 h-min bg-gray-400 text-black",
            )
            .set_attr("style", r#"resize:none"#)
            .build();

        let query_button = TnButton::builder()
            .init(AGENT_QUERY_BUTTON.into(), "Send".into())
            .set_attr(
                "class",
                "btn btn-xs btn-outline btn-primary w-full h-min p-1 join-item",
            )
            .set_action(TnActionExecutionMethod::Await, query)
            .build();

        let asset_search_button = TnButton::builder()
            .init(ASSET_SEARCH_BUTTON.into(), "Search Asset".into())
            .set_attr(
                "class",
                "btn btn-xs btn-outline btn-primary w-full h-min p-1 join-item",
            )
            .set_action(TnActionExecutionMethod::Await, search_asset_clicked)
            .build();

        let asset_search_output = TnDiv::builder()
        .init(ASSET_SEARCH_OUTPUT.into(), "Asset Search Results\n".to_string())
        .set_attr("class", "flex flex-1 border-2 overflow-y-scroll scrollbar-thin text-wrap mb-1 border-gray-600 bg-gray-800 rounded-lg p-1 min-h-[520px] max-h-[520px] min-w-[140px] max-w-[280px]")
        .build();

        let top_k_slider = TnRangeSlider::builder()
            .init(TOPK_SLIDER.into(), 8.0, 4.0, 16.0)
            .set_attr("class", "flex flex-1 min-w-[140px] max-w-[280px]")
            .set_action(TnActionExecutionMethod::Await, top_k_value_update)
            .build();

        let threshold_slider = TnRangeSlider::builder()
            .init(THRESHOLD_SLIDER.into(), 65.0, 60.0, 80.0)
            .set_attr("class", "flex flex-1 min-w-[140px] max-w-[280px]")
            .set_action(TnActionExecutionMethod::Await, threshold_value_update)
            .build();

        let temperature_slider = TnRangeSlider::builder()
            .init(TEMPERATURE_SLIDER.into(), 5.0, 0.0, 100.0)
            .set_attr("class", "flex flex-1 min-w-[140px] max-w-[280px]")
            .set_action(TnActionExecutionMethod::Await, temperature_value_update)
            .build();

        // let new_session_button = TnButton::builder()
        //     .init(
        //         AGENT_NEW_SESSION_BUTTON.into(),
        //         "Start A New Session".into(),
        //     )
        //     .set_attr(
        //         "class",
        //         "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
        //     )
        //     // .set_action(TnActionExecutionMethod::Await, query_with_hits)
        //     .build();

        context.add_component(chat_textarea);
        context.add_component(query_text_input);
        context.add_component(agent_stream_output);
        context.add_component(query_button);
        context.add_component(asset_search_button);
        context.add_component(asset_search_output);
        context.add_component(top_k_slider);
        context.add_component(threshold_slider);
        context.add_component(temperature_slider);
        //context.add_component(new_session_button);

        self
    }
}

use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{markdown_to_html_with_plugins, Options, Plugins};

static SYNTECT_ADAPTER: OnceLock<SyntectAdapter> = OnceLock::new();
static COMRAK_PLUGINS: OnceLock<Plugins> = OnceLock::new();

pub fn get_comrak_plugins() -> &'static Plugins<'static> {
    COMRAK_PLUGINS.get_or_init(|| {
        let syntect_adapter = SYNTECT_ADAPTER.get_or_init(|| {
            SyntectAdapterBuilder::new()
                .theme("base16-ocean.light")
                .build()
        });
        let mut comrak_plugins = Plugins::default();
        comrak_plugins.render.codefence_syntax_highlighter = Some(syntect_adapter);
        comrak_plugins
    })
}

#[async_trait]
impl<'a> TnComponentRenderTrait<'a> for AgentWorkSpace<'a>
where
    'a: 'static,
{
    /// Generates the internal HTML representation of the button component.
    async fn render(&self) -> String {
        self.html.clone()
    }

    /// Generates the initial HTML representation of the button component.
    async fn initial_render(&self) -> String {
        self.render().await
    }

    async fn pre_render(&mut self, ctx: &TnContextBase) {
        let comp_guard = ctx.components.read().await;

        let stream_output_html = comp_guard
            .get(AGENT_STREAM_OUTPUT)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let query_text_input_html = comp_guard
            .get(AGENT_QUERY_TEXT_INPUT)
            .unwrap()
            .read()
            .await
            .render()
            .await;

        let query_button_html = comp_guard
            .get(AGENT_QUERY_BUTTON)
            .unwrap()
            .read()
            .await
            .render()
            .await;

        let asset_search_button_html = comp_guard
            .get(ASSET_SEARCH_BUTTON)
            .unwrap()
            .read()
            .await
            .render()
            .await;

        // let new_session_button_html = comp_guard
        //     .get(AGENT_NEW_SESSION_BUTTON)
        //     .unwrap()
        //     .read()
        //     .await
        //     .render()
        //     .await;

        let agent_name = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::String(s)) = assets_guard.get("agent_name") {
                s.clone()
            } else {
                "Chat".into()
            }
        };

        let agent_id = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::U32(s)) = assets_guard.get("agent_id") {
                *s
            } else {
                0
            }
        };

        let chat_id = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::U32(chat_id)) = assets_guard.get("chat_id") {
                *chat_id as i32
            } else {
                tracing::info!(target: TRON_APP, "creating a chat workspace, no chid_id found");
                panic!("no chat id found")
            }
        };

        let asset_id = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::U32(asset_id)) = assets_guard.get("asset_id") {
                *asset_id as i32
            } else {
                tracing::info!(target: TRON_APP, "creating a chat workspace, no asset_id found");
                panic!("no asset id found")
            }
        };

        let messages = get_messages(chat_id).await.unwrap_or_default();

        {
            let chat_textarea = comp_guard.get(AGENT_CHAT_TEXTAREA).unwrap();
            let comrak_options = Options::default();
            let comrak_plugins = get_comrak_plugins();
            chatbox::clean_chatbox_value(chat_textarea.clone()).await;
            for (role, _m_type, content) in messages.into_iter() {
                match role.as_str() {
                    "bot" => {
                        let html_output = [
                            r#"<article class="markdown-body bg-blue-900 text-gray-200 p-3">"#
                                .to_string(),
                            markdown_to_html_with_plugins(
                                &content,
                                &comrak_options,
                                comrak_plugins,
                            ),
                            r#"<article>"#.to_string(),
                        ]
                        .join("\n");
                        chatbox::append_chatbox_value(chat_textarea.clone(), (role, html_output))
                            .await;
                    }
                    _ => {
                        chatbox::append_chatbox_value(
                            chat_textarea.clone(),
                            (role, ammonia::clean_text(&content)),
                        )
                        .await;
                    }
                }
            }
        }

        let chat_textarea_html = comp_guard
            .get(AGENT_CHAT_TEXTAREA)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let asset_search_output_html = comp_guard
            .get(ASSET_SEARCH_OUTPUT)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let topk_slider_html = comp_guard
            .get(TOPK_SLIDER)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let threshold_slider_html = comp_guard
            .get(THRESHOLD_SLIDER)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let temperature_slider_html = comp_guard
            .get(TEMPERATURE_SLIDER)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        self.html = AgentWorkspaceTemplate {
            agent_name,
            agent_id,
            chat_id,
            asset_id,
            chat_textarea: chat_textarea_html,
            stream_output: stream_output_html,
            query_text_input: query_text_input_html,
            asset_search_button: asset_search_button_html,
            query_button: query_button_html,
            asset_search_output: asset_search_output_html,
            topk_slider_html,
            threshold_slider_html,
            temperature_slider_html,
        }
        .render()
        .unwrap()
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}

async fn insert_message(
    chat_id: i32,
    user_id: i32,
    agent_id: i32,
    content: &str,
    role: &str,
    message_type: &str,
    fsm_state: Option<String>,
) -> Result<i32, sqlx::Error> {
    let pool = DB_POOL.clone();
    let result = sqlx::query!(
        r#"
        INSERT INTO messages (chat_id, user_id, agent_id, content, role, message_type, fsm_state)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING message_id
        "#,
        chat_id,
        user_id,
        agent_id,
        content,
        role,
        message_type,
        fsm_state
    )
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query!(
        r#"
        UPDATE chats 
        SET last_fsm_state = $1
        WHERE chat_id = $2
        "#,
        fsm_state,
        chat_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(result.message_id)
}

async fn get_messages(chat_id: i32) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let pool = DB_POOL.clone();
    let results = sqlx::query!(
        r#"
        SELECT role, message_type, content
        FROM messages
        WHERE chat_id = $1 
        ORDER BY timestamp ASC
        "#,
        chat_id
    )
    .fetch_all(&pool)
    .await?;

    let messages: Vec<(String, String, String)> = results
        .into_iter()
        .map(|row| (row.role.unwrap_or_default(), row.message_type, row.content))
        .collect::<Vec<_>>();
    Ok(messages)
}

async fn get_chat_summary(chat_id: i32) -> Result<String, sqlx::Error> {
    let pool = DB_POOL.clone();
    let result = sqlx::query!(r#" SELECT summary FROM chats WHERE chat_id = $1 "#, chat_id)
        .fetch_one(&pool)
        .await?;
    let summary = result.summary.unwrap_or_default();

    Ok(summary)
}

async fn update_chat_summary(chat_id: i32, summary: &str) -> Result<i32, sqlx::Error> {
    let pool = DB_POOL.clone();
    let result = sqlx::query!(
        r#"
        UPDATE chats 
        SET summary = $1, updated_at = CURRENT_TIMESTAMP 
        WHERE chat_id = $2 
        RETURNING chat_id
        "#,
        summary,
        chat_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(result.chat_id)
}

fn query(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    // TODO: this event handler needs significant refactoring
    tn_future! {
        if event.e_trigger != AGENT_QUERY_BUTTON {
            return None;
        };

        // spawn a handler for processing the output of the stream service
        let (tx, mut rx) = mpsc::channel::<(String, String)>(8);
        let context_cloned = context.clone();
        let handle = tokio::spawn(async move {

            let comrak_options = Options::default();
            let comrak_plugins = get_comrak_plugins();
            while let Some((t, r)) = rx.recv().await {
                match t.as_str() {
                    "token" =>  {
                    // tracing::info!(target: "tron_app", "streaming token: {}", r);
                    text::append_and_update_stream_textarea_with_context(
                        &context_cloned,
                        AGENT_STREAM_OUTPUT,
                        &r,
                    )
                    .await},
                    "llm_output" => {
                        let query_result_area = context_cloned.get_component(AGENT_CHAT_TEXTAREA).await;
                        let html_output = [
                            r#"<article class="markdown-body bg-blue-900 text-gray-200 p-3">"#.to_string(),
                            markdown_to_html_with_plugins(&r, &comrak_options, comrak_plugins),
                            r#"<article>"#.to_string(),
                        ]
                        .join("\n");
                        chatbox::append_chatbox_value(query_result_area.clone(), ("bot".into(), html_output)).await;
                        context_cloned.set_ready_for(AGENT_CHAT_TEXTAREA).await;
                    },
                    "clear" => {
                        text::clean_stream_textarea_with_context(
                            &context_cloned,
                            AGENT_STREAM_OUTPUT,
                        )
                        .await;
                    }
                    "message" => {
                        let message = format!("\nLLM Engine Message: {}", r);
                        text::append_and_update_stream_textarea_with_context(
                            &context_cloned,
                            AGENT_STREAM_OUTPUT,
                            &message,
                        )
                        .await
                    }
                    _ => {}
                }
            }
        });

        let query_text = context.get_value_from_component(AGENT_QUERY_TEXT_INPUT).await;

        let query_text = if let TnComponentValue::String(query_text) = query_text {
            query_text
        } else {
            return None
        };

        if query_text.is_empty() {
            return None
        }

        let fsm_agent_config;
        let llm_name;
        let user_id;
        let agent_id;
        let chat_id;
        let asset_id;

        {
            let asset_ref = context.get_asset_ref().await;
            let asset_guard = asset_ref.read().await;

            //let provider;
            let _agent_config = if let TnAsset::String(agent_config) = asset_guard.get("agent_configuration").unwrap() {
                let model_setting: AgentSetting =
                    serde_json::from_str::<AgentSetting>(agent_config).unwrap();
                    llm_name = model_setting.model_name;
                    //provider = model_setting.provider;
                    fsm_agent_config = model_setting.fsm_agent_config;
                agent_config
            } else {
                llm_name = "gpt-4o".into();
                //provider = "OpenAI".into();
                fsm_agent_config = "{}".into();
                &"".into()
            };

            user_id = if let TnAsset::U32(user_id) =  asset_guard.get("user_id").unwrap() {
                *user_id as i32
            } else {
                panic!("chat_id not found");
            };

            agent_id = if let TnAsset::U32(agent_id) =  asset_guard.get("agent_id").unwrap() {
                *agent_id as i32
            } else {
                panic!("chat_id not found");
            };

            chat_id = if let TnAsset::U32(chat_id) =  asset_guard.get("chat_id").unwrap() {
                *chat_id as i32
            } else {
                panic!("chat_id not found");
            };

            asset_id = if let TnAsset::U32(chat_id) =  asset_guard.get("asset_id").unwrap() {
                *chat_id as i32
            } else {
                panic!("chat_id not found");
            };
        }

        let fsm_config = FSMAgentConfigBuilder::from_toml(&fsm_agent_config).unwrap().build().unwrap();

        let fsm = FSMBuilder::from_config::<FSMChatState>(&fsm_config, HashMap::default()).unwrap().build().unwrap();

        let api_key = match llm_name.as_str() {
            "gpt-4o" | "gpt-4o-mini" | "gpt-3.5-turbo" | "o3-mini" => {
                if let Ok(open_api_key) = std::env::var("OPENAI_API_KEY") {
                    open_api_key
                } else {
                    let mut h = HeaderMap::new();
                    h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
                    h.insert("Hx-Retarget", "#env_var_setting_notification_msg".parse().unwrap());
                    h.insert("HX-Trigger-After-Swap", "show_env_var_setting_notification".parse().unwrap());

                    return Some(
                        (h, Html::from("Environment variable OPENAI_API_KEY not set for the agent for the server, please set it up, restart the server, and reload the web app.".to_string())) );
                }},
            "claude-3-haiku-20240307" | "claude-3-5-sonnet-20241022" => {
                if let Ok(open_api_key) = std::env::var("ANTHROPIC_API_KEY") {
                    open_api_key
                } else {
                    let mut h = HeaderMap::new();
                    h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
                    h.insert("Hx-Retarget", "#env_var_setting_notification_msg".parse().unwrap());
                    h.insert("HX-Trigger-After-Swap", "show_env_var_setting_notification".parse().unwrap());

                    return Some(
                        (h, Html::from("Environment variable ANTHROPIC_API_KEY not set for the agent for the server, please set it up, restart the server, and reload the web app.".to_string())) );
                }},
            _ => {"".into()}

        };


        let (fsm_state, exec_entry_actions) = {
            let asset = context.get_asset_ref().await;
            let asset_guard = asset.read().await;
            if let Some(TnAsset::String(fsm_state)) = asset_guard.get("fsm_state") {
                (Some(fsm_state.clone()), false)
            } else {
                (fsm.current_state(), true) // default initial state from fsm_config
            }
        };

        let agent_settings = AgentSettings {
            sys_prompt: fsm_config.sys_prompt,
            fsm_prompt: fsm_config.fsm_prompt,
            summary_prompt: fsm_config.summary_prompt,
            model: llm_name.clone(),
            api_key,
        };

        let mut agent = LLMAgent::new(fsm, agent_settings); // we start a new agent every query now, we may want to implement session/static agent
        {
            if let Err(_e) = agent.set_current_state(fsm_state.clone(), exec_entry_actions).await {
                let fsm_state = agent.fsm.current_state();
                agent.set_current_state(fsm_state, true).await.expect("set current state fail");
            };
            agent.llm_req_settings.summary = get_chat_summary(chat_id).await.unwrap_or_default();
            // we may want to load a couple of last message from the database for the agent providing some memory beyond the summary
        }

        {
            let asset = context.get_asset_ref().await;
            let asset_guard = asset.read().await;
            let top_k: u32 = if let Some(TnAsset::U32(top_k)) = asset_guard.get("top_k_value") {
                *top_k
            } else {
                8
            };
            let threshold_value: f32 = if let Some(TnAsset::F32(threshold_value)) = asset_guard.get("threshold_value") {
                *threshold_value
            } else {
                0.65
            };

            let temperature_value: Option<f32> = if let Some(TnAsset::F32(temperature)) = asset_guard.get("temperature_value") {
                Some(*temperature)
            } else {
                None
            };


            let search_asset_results = search_asset(&query_text, asset_id, top_k as usize, threshold_value).await;

            let query_context = get_search_context_plain_text(&search_asset_results);

            agent.llm_req_settings.context = Some(query_context);

            let query_context_html = get_search_context_html(&search_asset_results);

            clean_div_with_context(
                &context,
                ASSET_SEARCH_OUTPUT,
            ).await;
            update_and_send_div_with_context(&context, ASSET_SEARCH_OUTPUT, &query_context_html).await;

            context.set_ready_for(AGENT_QUERY_TEXT_INPUT).await;
            clean_textarea_with_context(
                &context,
                AGENT_QUERY_TEXT_INPUT,
            )
            .await;

            let query_result_area = context.get_component(AGENT_CHAT_TEXTAREA).await;
            chatbox::append_chatbox_value(query_result_area.clone(), ("user".into(), ammonia::clean_text(&query_text))).await;
            context.set_ready_for(AGENT_CHAT_TEXTAREA).await;
            let _ = insert_message(chat_id, user_id, agent_id, &query_text, "user", "text", None).await;

            match agent.process_message(&query_text, Some(tx), temperature_value).await {
                Ok(res) => {
                    let current_state = agent.get_current_state().await;
                    let _ = insert_message(chat_id, user_id, agent_id, &res, "bot", "text", current_state).await;
                    let _ = update_chat_summary(chat_id, &agent.llm_req_settings.summary).await;
                },
                Err(err) => {
                    tracing::info!(target: "tron_app", "LLM API call error: {:?}", err);

                    let mut h = HeaderMap::new();
                    h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
                    h.insert("Hx-Retarget", "#env_var_setting_notification_msg".parse().unwrap());
                    h.insert("HX-Trigger-After-Swap", "show_env_var_setting_notification".parse().unwrap());

                    return Some(
                        (h, Html::from(format!(r#"LLM ({}) API Call fail, please check the API key is set and correct. You may need to restart the server and reload the app with the correct API key(s)."#, llm_name))));

                }
            };
        }


        handle.abort();
        {
            let asset = context.get_asset_ref().await;
            let mut asset_guard = asset.write().await;
            asset_guard.insert("fsm_state".into(), TnAsset::String(agent.fsm.current_state().unwrap()) );
        }


        None
    }
}

async fn search_asset(query: &str, asset_id: i32, top_k: usize, threshold: f32) -> Vec<TwoDPoint> {
    if asset_id == 0 {
        return vec![];
    };

    // use LLM to extend the context for simple question
    // let query = &extend_query_with_llm(query).await;
    // tracing::info!(target: TRON_APP, "extended query: {}", query);

    let tk_service = TextChunkingService::new(None, 128, 0, 4096);

    let mut chunks = tk_service.text_to_chunks(query);

    // tracing::info!(target:"tron_app", "chunks: {:?}", chunks);
    EMBEDDING_SERVICE
        .get()
        .unwrap()
        .get_embedding_for_chunks(&mut chunks)
        .expect("Failed to get embeddings");
    let mut min_d = OrderedFloat::from(f64::MAX);
    let mut best_sorted_points = Vec::<TwoDPoint>::new();
    for c in chunks.into_iter() {
        let ev = c.embedding_vec.unwrap().clone();
        let sorted_points =
            vector_query_and_sort_points(asset_id, &ev, Some(top_k as i32), Some(threshold)).await;
        if sorted_points.is_empty() {
            break;
        };
        let d = sorted_points.first().unwrap().d;
        if d < min_d {
            min_d = d;
            best_sorted_points = sorted_points;
        }
    }

    let top_k = if top_k > best_sorted_points.len() {
        best_sorted_points.len()
    } else {
        top_k
    };
    let top_hits: Vec<TwoDPoint> = best_sorted_points[..top_k].into();
    top_hits
}

fn get_search_context_plain_text(top_hits: &[TwoDPoint]) -> String {
    top_hits
        .iter()
        .map(|p| {
            let c = &p.chunk;
            format!(
                "<DOCUMENT>\n<DOCUMET_TITLE>{}</DOCUMENT_TITLE>\n\n<CONTEXT>\n{}\n</CONTEXT>\n</DOCUMENT>",
                c.title,
                c.text
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_search_context_html(top_hits: &[TwoDPoint]) -> String {
    let md_text = top_hits
        .iter()
        .map(|p| {
            let c = &p.chunk;
            let context = c
                .text
                .split("\n")
                .map(|s| format!("> {}", s))
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                "### {}\n Similarity: {:0.5} \n\n CONTEXT:\n{}",
                c.title,
                1.0 - p.d.to_f64(),
                context
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");
    let mut comrak_options = Options::default();
    comrak_options.render.width = 20;
    let comrak_plugins = get_comrak_plugins();
    [
        r#"<article class="flex flex-1 flex-col markdown-body bg-gray-800 text-gray-100 min-w-[140px] max-w-[280px]">"#.to_string(),
        markdown_to_html_with_plugins(&md_text, &comrak_options, comrak_plugins),
        r#"<article>"#.to_string(),
    ]
    .join("\n")
}

async fn extend_query_with_llm(query: &str) -> String {
    let llm_name = "gpt-3.5-turbo";
    let api_key = match llm_name {
        "gpt-4o" | "gpt-4o-mini" | "gpt-3.5-turbo" | "o3-mini" => std::env::var("OPENAI_API_KEY")
            .map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
                env_name: "OPENAI_API_KEY".to_string(),
            })
            .unwrap(),
        "claude-3-haiku-20240307" | "claude-3-5-sonnet-20241022" => {
            std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
                    env_name: "ANTHROPIC_API_KEY".to_string(),
                })
                .unwrap()
        }
        _ => "".into(),
    };

    let llm_client = GenaiLlmclient {
        model: llm_name.to_string(),
        api_key,
    };
    let prompt = "find the relevant information about the questions and summary it into a small response less in 100 words.";
    llm_client
        .generate(prompt, &[("user".into(), query.into())], Some(0.05))
        .await
        .unwrap()
}

fn search_asset_clicked(
    context: TnContext,
    event: TnEvent,
    _payload: Value,
) -> TnFutureHTMLResponse {
    tn_future! {
        if event.e_trigger != ASSET_SEARCH_BUTTON {
            return None;
        };

        let asset_ref = context.get_asset_ref().await;
        let asset_guard = asset_ref.read().await;

        // let _user_id = if let TnAsset::U32(user_id) =  asset_.get("user_id").unwrap() {
        //     *user_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        // let _agent_id = if let TnAsset::U32(agent_id) =  asset_guard.get("agent_id").unwrap() {
        //     *agent_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        // let _chat_id = if let TnAsset::U32(chat_id) =  asset_guard.get("chat_id").unwrap() {
        //     *chat_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        let asset_id = if let TnAsset::U32(chat_id) =  asset_guard.get("asset_id").unwrap() {
            *chat_id as i32
        } else {
            panic!("chat_id not found");
        };

        let query_text = context.get_value_from_component(AGENT_QUERY_TEXT_INPUT).await;

        let query_text = if let TnComponentValue::String(s) = query_text {
            s
        } else {
            unreachable!()
        };

        let query_text = {
            if query_text.len() > 5 {
                query_text.clone()
            } else {
                return None;
            }
        };

        let asset = context.get_asset_ref().await;
        let asset_guard = asset.read().await;
        let top_k: u32 = if let Some(TnAsset::U32(top_k)) = asset_guard.get("top_k_value") {
            *top_k
        } else {
            8
        };
        let threshold_value: f32 = if let Some(TnAsset::F32(top_k)) = asset_guard.get("threshold_value") {
            *top_k
        } else {
            0.75
        };

        // tracing::info!(target:"tron_app", "query_text: {}", query_text);

        let search_asset_results = search_asset(&query_text, asset_id, top_k as usize, threshold_value).await;

        let query_context = get_search_context_html(&search_asset_results);
        clean_div_with_context(
            &context,
            ASSET_SEARCH_OUTPUT,
        ).await;
        update_and_send_div_with_context(&context, ASSET_SEARCH_OUTPUT, &query_context).await;
        None
    }
}

fn top_k_value_update(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        let slider = context.get_component(&event.e_trigger).await;
        let value = if let TnComponentValue::String(value) = slider.read().await.value() {
            let new_str = format!("{} -- Value {};", event.e_trigger, value);
            tracing::info!(target: TRON_APP, "{}", new_str);
            let asset = context.get_asset_ref().await;
            let mut asset_guard = asset.write().await;
            let v = value.parse::<u32>().unwrap();
            asset_guard.insert("top_k_value".into(), TnAsset::U32(v));
            value.to_string()

        } else {
            "error".into()
        };
        let mut h = HeaderMap::new();
        h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
        h.insert("Hx-Retarget", "#top_k_value".parse().unwrap());
        Some((h, Html::from(value)))
    }
}

fn threshold_value_update(
    context: TnContext,
    event: TnEvent,
    _payload: Value,
) -> TnFutureHTMLResponse {
    tn_future! {
        let slider = context.get_component(&event.e_trigger).await;
        let value = if let TnComponentValue::String(value) = slider.read().await.value() {
            let new_str = format!("{} -- Value {};", event.e_trigger, value);
            tracing::info!(target: TRON_APP, "{}", new_str);
            let v = value.parse::<f32>().unwrap() / 100.0;
            let asset = context.get_asset_ref().await;
            let mut asset_guard = asset.write().await;
            asset_guard.insert("threshold_value".into(), TnAsset::F32(v));
            format!("{:0.2}", v)
        } else {
            "error".into()
        };
        let mut h = HeaderMap::new();
        h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
        h.insert("Hx-Retarget", "#threshold_value".parse().unwrap());
        Some((h, Html::from(value)))
    }
}

fn temperature_value_update(
    context: TnContext,
    event: TnEvent,
    _payload: Value,
) -> TnFutureHTMLResponse {
    tn_future! {
        let slider = context.get_component(&event.e_trigger).await;
        let value = if let TnComponentValue::String(value) = slider.read().await.value() {
            let new_str = format!("{} -- Value {};", event.e_trigger, value);
            tracing::info!(target: TRON_APP, "{}", new_str);
            let v = value.parse::<f32>().unwrap() / 100.0;
            let asset = context.get_asset_ref().await;
            let mut asset_guard = asset.write().await;
            asset_guard.insert("temperature_value".into(), TnAsset::F32(v));
            format!("{:0.2}", v)
        } else {
            "error".into()
        };
        let mut h = HeaderMap::new();
        h.insert("Hx-Reswap", "innerHTML".parse().unwrap());
        h.insert("Hx-Retarget", "#temperature_value".parse().unwrap());
        Some((h, Html::from(value)))
    }
}
