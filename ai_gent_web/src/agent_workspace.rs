use ai_gent_lib::fsm::FSMBuilder;
use ai_gent_lib::llm_agent::FSMAgentConfigBuilder;
use ai_gent_lib::llm_agent::LLMAgent;
use askama::Template;
use async_trait::async_trait;
use axum::http::HeaderMap;
use axum::response::Html;
use candle_core::WithDType;
use html_escape::encode_text;
use ordered_float::OrderedFloat;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tron_app::tron_components::text::append_textarea_value;
use tron_app::tron_components::text::update_and_send_textarea_with_context;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;
use tron_app::TRON_APP;

use super::DB_POOL;
use crate::embedding_service::vector_query_and_sort_points;
use crate::embedding_service::TextChunkingService;
use crate::embedding_service::TwoDPoint;
use crate::llm_agent::*;
use crate::AgentSetting;
use crate::SEARCH_AGENT_BTN;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::Acquire;
use sqlx::Postgres;
use sqlx::{any::AnyRow, prelude::FromRow, query as sqlx_query};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use crate::embedding_service::EMBEDDING_SERVICE;
use pulldown_cmark::{html, Options, Parser};

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
                "flex-1 border-2 mb-1 border-gray-600 rounded-lg p-1 h-min bg-gray-400 text-black",
            )
            .set_attr("style", r#"resize:none"#)
            .build();

        let query_button = TnButton::builder()
            .init(AGENT_QUERY_BUTTON.into(), "Send".into())
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
            )
            .set_action(TnActionExecutionMethod::Await, query)
            .build();

        let asset_search_button = TnButton::builder()
            .init(ASSET_SEARCH_BUTTON.into(), "Search Asset".into())
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
            )
            .set_action(TnActionExecutionMethod::Await, search_asset_clicked)
            .build();

        let asset_search_output = text::TnTextArea::builder()
        .init(ASSET_SEARCH_OUTPUT.into(), "Asset Search Results\n".to_string())
        .set_attr("class", "flex-1 border-2 overflow-x-scroll text-nowrap mb-1 border-gray-600 bg-gray-400 text-black rounded-lg p-1 min-h-[60svh] w-full")
        .build();

        let topk_slider = TnRangeSlider::builder()
            .init(TOPK_SLIDER.into(), 8.0, 4.0, 16.0)
            .set_attr("class", "flex-1 ml-auto w-full")
            .set_action(TnActionExecutionMethod::Await, top_k_value_update)
            .build();

        let threshold_slider = TnRangeSlider::builder()
            .init(THRESHOLD_SLIDER.into(), 75.0, 60.0, 100.0)
            .set_attr("class", "flex-1 ml-auto w-full")
            .set_action(TnActionExecutionMethod::Await, threshold_value_update)
            .build();

        let temperature_slider = TnRangeSlider::builder()
            .init(TEMPERATURE_SLIDER.into(), 5.0, 0.0, 50.0)
            .set_attr("class", "flex-1 ml-auto w-full")
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
        context.add_component(topk_slider);
        context.add_component(threshold_slider);
        context.add_component(temperature_slider);
        //context.add_component(new_session_button);

        self
    }
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
                panic!("no chat id found")
            }
        };

        let asset_id = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::U32(asset_id)) = assets_guard.get("asset_id") {
                *asset_id as i32
            } else {
                panic!("no asset id found")
            }
        };

        let messages = get_messages(chat_id).await.unwrap_or_default();

        {
            let chat_textarea = comp_guard.get(AGENT_CHAT_TEXTAREA).unwrap();
            chatbox::clean_chatbox_value(chat_textarea.clone()).await;
            for (role, _m_type, content) in messages.into_iter() {
                match role.as_str() {
                    "bot" => {
                        let parser = Parser::new_ext(&content, Options::all());
                        let mut html_output = String::new();
                        html::push_html(&mut html_output, parser);
                        chatbox::append_chatbox_value(chat_textarea.clone(), (role, html_output))
                            .await;
                    }
                    _ => {
                        chatbox::append_chatbox_value(chat_textarea.clone(), (role, content)).await;
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
) -> Result<i32, sqlx::Error> {
    let pool = DB_POOL.clone();
    let result = sqlx::query!(
        r#"
        INSERT INTO messages (chat_id, user_id, agent_id, content, role, message_type)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING message_id
        "#,
        chat_id,
        user_id,
        agent_id,
        content,
        role,
        message_type
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

//const FSM_PROMPT: &str = include_str!("../dev_config/fsm_prompt"); // this should be generated from fsm_agent_config
//const SUMMARY_PROMPT: &str = include_str!("../dev_config/summary_prompt");
fn query(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        if event.e_trigger != AGENT_QUERY_BUTTON {
            return None;
        };

        let query_text = context.get_value_from_component(AGENT_QUERY_TEXT_INPUT).await;

        let query_text = if let TnComponentValue::String(query_text) = query_text {
            encode_text(&query_text).to_string()
        } else {
            return None
        };

        if query_text.is_empty() {
            return None
        }

        let asset_ref = context.get_asset_ref().await;
        let asset_guarad = asset_ref.read().await;
        let fsm_agent_config;
        let llm_name;
        //let provider;
        let _agent_config = if let TnAsset::String(agent_config) = asset_guarad.get("agent_configuration").unwrap() {
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

        let user_id = if let TnAsset::U32(user_id) =  asset_guarad.get("user_id").unwrap() {
            *user_id as i32
        } else {
            panic!("chat_id not found");
        };

        let agent_id = if let TnAsset::U32(agent_id) =  asset_guarad.get("agent_id").unwrap() {
            *agent_id as i32
        } else {
            panic!("chat_id not found");
        };

        let chat_id = if let TnAsset::U32(chat_id) =  asset_guarad.get("chat_id").unwrap() {
            *chat_id as i32
        } else {
            panic!("chat_id not found");
        };

        let asset_id = if let TnAsset::U32(chat_id) =  asset_guarad.get("asset_id").unwrap() {
            *chat_id as i32
        } else {
            panic!("chat_id not found");
        };



        let fsm_config = FSMAgentConfigBuilder::from_toml(&fsm_agent_config).unwrap().build().unwrap();

        let fsm = FSMBuilder::from_config(&fsm_config).unwrap().build().unwrap();

        let api_key = match llm_name.as_str() {
            "gpt-4o" | "gpt-4o-mini" | "gpt-3.5-turbo" | "o3-mini" => { std::env::var("OPENAI_API_KEY").map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
                env_name: "OPENAI_API_KEY".to_string()}).unwrap() },
            "claude-3-haiku-20240307" | "claude-3-5-sonnet-20241022" => { std::env::var("ANTHROPIC_API_KEY").map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
                env_name: "ANTHROPIC_API_KEY".to_string()}).unwrap()
            },
            _ => {"".into()}

        };
            std::env::var("OPENAI_API_KEY").map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
            env_name: "OPENAI_API_KEY".to_string()}).unwrap();

        let llm_client = GenaiLlmclient {
            model: llm_name,
            api_key
        };

        let mut agent = LLMAgent::new(fsm, llm_client, &fsm_config);

        agent.summary = get_chat_summary(chat_id).await.unwrap_or_default();

        // let mut options = Options::empty();
        // options.insert(Options::ENABLE_STRIKETHROUGH);

        let (tx, mut rx) = mpsc::channel::<(String, String)>(8);
        let context_cloned = context.clone();
        let handle = tokio::spawn(async move {
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
                        // tracing::info!(target: "tron_app", "LLM Response: {}", r);
                        let parser = Parser::new_ext(&r, Options::all());
                        let mut html_output = String::new();
                        html::push_html(&mut html_output, parser);
                        let query_result_area = context_cloned.get_component(AGENT_CHAT_TEXTAREA).await;
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
                    "fsm_state" => {
                        let fsm_state = format!("\nFSM State: {}", r);
                        text::append_and_update_stream_textarea_with_context(
                            &context_cloned,
                            AGENT_STREAM_OUTPUT,
                            &fsm_state,
                        )
                        .await
                    }
                    _ => {}
                }
            }
        });

        {
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


            let query_context = encode_text(&search_asset(&query_text, asset_id, top_k as usize, threshold_value).await).to_string();
            text::clean_textarea_with_context(
                &context,
                ASSET_SEARCH_OUTPUT,
            ).await;
            update_and_send_textarea_with_context(&context, ASSET_SEARCH_OUTPUT, &query_context).await;
            agent.context = Some(query_context);

            context.set_ready_for(AGENT_QUERY_TEXT_INPUT).await;
            text::clean_textarea_with_context(
                &context,
                AGENT_QUERY_TEXT_INPUT,
            )
            .await;
            let query_result_area = context.get_component(AGENT_CHAT_TEXTAREA).await;
            chatbox::append_chatbox_value(query_result_area.clone(), ("user".into(), query_text.to_string())).await;
            context.set_ready_for(AGENT_CHAT_TEXTAREA).await;
            let _ = insert_message(chat_id, user_id, agent_id, &query_text, "user", "text").await;
            match agent.process_message(&query_text, Some(tx)).await {
                Ok(res) => {
                    let _ = insert_message(chat_id, user_id, agent_id, &res, "bot", "text").await;
                    let _ = update_chat_summary(chat_id, &agent.summary).await;
                },
                Err(err) => tracing::info!(target: "tron_app", "LLM error, please retry your question. {:?}", err),
            };
        }

        handle.abort();


        None
    }
}

async fn search_asset(query: &str, asset_id: i32, top_k: usize, threshold: f32) -> String {
    if asset_id == 0 {
        return "".to_string();
    };

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
            break
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
    let out = top_hits
        .iter()
        .map(|p| {
            let c = &p.chunk;
            format!(
                "DOCUMET TITLE:\n{}\n\n (Similarity: {:0.5}, span: {}-{}) \n\nCONTEXT:\n{}",
                c.title,
                1.0 - p.d.to_f64(),
                c.span.0,
                c.span.1,
                c.text
            )
        })
        .collect::<Vec<String>>()
        .join("\n========================\n\n");

    out
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
        let asset_guarad = asset_ref.read().await;

        // let _user_id = if let TnAsset::U32(user_id) =  asset_guarad.get("user_id").unwrap() {
        //     *user_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        // let _agent_id = if let TnAsset::U32(agent_id) =  asset_guarad.get("agent_id").unwrap() {
        //     *agent_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        // let _chat_id = if let TnAsset::U32(chat_id) =  asset_guarad.get("chat_id").unwrap() {
        //     *chat_id as i32
        // } else {
        //     panic!("chat_id not found");
        // };

        let asset_id = if let TnAsset::U32(chat_id) =  asset_guarad.get("asset_id").unwrap() {
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
        let out = search_asset(&query_text, asset_id, top_k as usize, threshold_value).await;
        text::clean_textarea_with_context(
            &context,
            ASSET_SEARCH_OUTPUT,
        ).await;
        update_and_send_textarea_with_context(&context, ASSET_SEARCH_OUTPUT, &out).await;
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
