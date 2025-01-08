use ai_gent_lib::fsm::FSMBuilder;
use ai_gent_lib::llm_agent::FSMAgentConfigBuilder;
use ai_gent_lib::llm_agent::LLMAgent;
use askama::Template;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;

use super::DB_POOL;
use crate::llm_agent::*;
use crate::AgentSetting;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::Acquire;
use sqlx::Postgres;
use sqlx::{any::AnyRow, prelude::FromRow, query as sqlx_query};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use pulldown_cmark::{html, Options, Parser};

pub const AGENT_CHAT_TEXTAREA: &str = "agent_chat_textarea";
pub const AGENT_STREAM_OUTPUT: &str = "agent_stream_output";
pub const AGENT_QUERY_TEXT_INPUT: &str = "agent_query_text_input";
pub const AGENT_QUERY_BUTTON: &str = "agent_query_button";
pub const AGENT_NEW_SESSION_BUTTON: &str = "agent_new_session_button";

/// Represents a button component in a Tron application.
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
    chat_textarea: String,
    query_text_input: String,
    stream_output: String,
    query_button: String,
    new_session_button: String,
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
                "min-h-[435px] max-h-[435px] overflow-auto flex-1 p-2",
            )
            .build();

        let mut query_text_input = TnTextArea::builder()
            .init(AGENT_QUERY_TEXT_INPUT.into(), "".into())
            .set_attr("class", "min-h-32 w-full p-2")
            .set_attr("style", "resize:none")
            .set_attr("hx-trigger", "change, server_event")
            .set_attr("hx-vals", r##"js:{event_data:get_input_event(event)}"##)
            .build(); //over-ride the default as we need the value of the input text
        query_text_input.remove_attribute("disabled");

        let agent_stream_output = TnStreamTextArea::builder()
            .init(AGENT_STREAM_OUTPUT.into(), Vec::new())
            .set_attr("class", "min-h-20 w-full p-2")
            .set_attr("style", r#"resize:none"#)
            .build();

        let query_button = TnButton::builder()
            .init(AGENT_QUERY_BUTTON.into(), "Submit Query".into())
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
            )
            .set_action(TnActionExecutionMethod::Await, query)
            .build();

        let new_session_button = TnButton::builder()
            .init(
                AGENT_NEW_SESSION_BUTTON.into(),
                "Start A New Session".into(),
            )
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
            )
            // .set_action(TnActionExecutionMethod::Await, query_with_hits)
            .build();

        context.add_component(chat_textarea);
        context.add_component(query_text_input);
        context.add_component(agent_stream_output);
        context.add_component(query_button);
        context.add_component(new_session_button);

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

        let new_session_button_html = comp_guard
            .get(AGENT_NEW_SESSION_BUTTON)
            .unwrap()
            .read()
            .await
            .render()
            .await;

        let agent_name = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::String(s)) = assets_guard.get("agent_name") {
                s.clone()
            } else {
                "Chat".into()
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

        let messages = get_messages(chat_id).await.unwrap_or_default();

        {
            let chat_textarea = comp_guard.get(AGENT_CHAT_TEXTAREA).unwrap();
            chatbox::clean_chatbox_value(chat_textarea.clone()).await;
            for (role, _m_type, content) in messages.into_iter() {
                chatbox::append_chatbox_value(chat_textarea.clone(), (role, content)).await;
            }
        }

        let chat_textarea_html = comp_guard
            .get(AGENT_CHAT_TEXTAREA)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        self.html = AgentWorkspaceTemplate {
            agent_name,
            chat_textarea: chat_textarea_html,
            stream_output: stream_output_html,
            query_text_input: query_text_input_html,
            query_button: query_button_html,
            new_session_button: new_session_button_html,
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
    println!("insert_message: chat:{}", chat_id);

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
    println!("message_id: {}", result.message_id);

    Ok(result.message_id)
}

async fn get_messages(chat_id: i32) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    println!("insert_message: chat:{}", chat_id);

    let pool = DB_POOL.clone();
    let results = sqlx::query!(
        r#"
        SELECT role, message_type, content
        FROM messages
        WHERE chat_id = $1 
        "#,
        chat_id
    )
    .fetch_all(&pool)
    .await?;

    let messages: Vec<(String, String, String)> = results
        .into_iter()
        .map(|row| (row.role.unwrap_or_default(), row.message_type, row.content))
        .collect::<Vec<_>>();
    println!("messages: {:?}", messages);
    Ok(messages)
}

async fn get_chat_summary(chat_id: i32) -> Result<String, sqlx::Error> {
    println!("update_chat_summary: chat:{}", chat_id);

    let pool = DB_POOL.clone();
    let result = sqlx::query!(r#" SELECT summary FROM chats WHERE chat_id = $1 "#, chat_id)
        .fetch_one(&pool)
        .await?;
    let summary = result.summary.unwrap_or_default();

    Ok(summary)
}

async fn update_chat_summary(chat_id: i32, summary: &str) -> Result<i32, sqlx::Error> {
    println!("update_chat_summary: chat:{}", chat_id);

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
    println!("updated chat_id: {}", result.chat_id);

    Ok(result.chat_id)
}

const FSM_PROMPT: &str = include_str!("../dev_config/fsm_prompt"); // this should be generated from fsm_agent_config
const SUMMARY_PROMPT: &str = include_str!("../dev_config/summary_prompt");
fn query(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        if event.e_trigger != AGENT_QUERY_BUTTON {
            return None;
        };


        let asset_ref = context.get_asset_ref().await;
        let asset_guarad = asset_ref.read().await;
        let fsm_agent_config;
        let _agent_config = if let TnAsset::String(agent_config) = asset_guarad.get("agent_configuration").unwrap() {
            let model_setting: AgentSetting =
                serde_json::from_str::<AgentSetting>(agent_config).unwrap();
                fsm_agent_config = model_setting.fsm_agent_config;
            agent_config
        } else {
            fsm_agent_config = "".into();
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


        let query_text = context.get_value_from_component(AGENT_QUERY_TEXT_INPUT).await;

        let fsm_config = FSMAgentConfigBuilder::from_json(&fsm_agent_config).unwrap().build().unwrap();

        let fsm = FSMBuilder::from_config(&fsm_config).unwrap().build().unwrap();

        let llm_client = OAI_LLMClient {};
        let mut agent = LLMAgent::new(fsm, llm_client, &fsm_config.sys_prompt, FSM_PROMPT, SUMMARY_PROMPT);

        agent.summary = get_chat_summary(chat_id).await.unwrap_or_default();

        // let mut options = Options::empty();
        // options.insert(Options::ENABLE_STRIKETHROUGH);

        let (tx, mut rx) = mpsc::channel::<String>(8);
        let context_cloned = context.clone();
        let handle = tokio::spawn(async move {
            while let Some(token) = rx.recv().await {
                println!("stream out: {}", token);
                text::append_and_update_stream_textarea_with_context(
                    &context_cloned,
                    AGENT_STREAM_OUTPUT,
                    &token,
                )
                .await;
            }
        });

        if let TnComponentValue::String(s) = query_text {
                context.set_ready_for(AGENT_QUERY_TEXT_INPUT).await;
                text::clean_textarea_with_context(
                    &context,
                    AGENT_QUERY_TEXT_INPUT,
                )
                .await;
                let query_result_area = context.get_component(AGENT_CHAT_TEXTAREA).await;
                let query = s.replace('\n', "<br>");
                chatbox::append_chatbox_value(query_result_area.clone(), ("user".into(), query.clone())).await;
                let _ = insert_message(chat_id, user_id, agent_id, &query, "user", "text").await;
                context.set_ready_for(AGENT_CHAT_TEXTAREA).await;
                match agent.process_input(&query, Some(tx)).await {
                    Ok(res) => {
                        println!("Response: {}", res);
                        let parser = Parser::new_ext(&res, Options::all());
                        let mut html_output = String::new();
                        // TODO: the markdown to HTML seems to be very slow
                        html::push_html(&mut html_output, parser);
                        chatbox::append_chatbox_value(query_result_area.clone(), ("bot".into(), html_output)).await;
                        let _ = insert_message(chat_id, user_id, agent_id, &res, "bot", "text").await;
                        let _ = update_chat_summary(chat_id, &agent.summary).await;
                    },
                    Err(err) => println!("LLM error, please retry your question. {:?}", err),
                };
        }

        handle.abort();
        context.set_ready_for(AGENT_CHAT_TEXTAREA).await;
        text::clean_stream_textarea_with_context(
            &context,
            AGENT_STREAM_OUTPUT,
        )
        .await;

        None
    }
}
