use askama::Template;
use axum::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;

use super::DB_POOL;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::Acquire;
use sqlx::Postgres;
use sqlx::{Column, Row, TypeInfo, ValueRef};

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
            .set_attr("class", "min-h-[435px] max-h-[435px] overflow-auto flex-1 p-2")
            .build();

        let mut query_text_input = TnTextArea::builder()
            .init(AGENT_QUERY_TEXT_INPUT.into(), "".into())
            .set_attr("class", "min-h-32 w-full")
            .set_attr("style", "resize:none")
            .set_attr("hx-trigger", "change, server_event")
            .set_attr("hx-vals", r##"js:{event_data:get_input_event(event)}"##)
            .build(); //over-ride the default as we need the value of the input text
        query_text_input.remove_attribute("disabled");


        let agent_stream_output = TnStreamTextArea::builder()
        .init(AGENT_STREAM_OUTPUT.into(), Vec::new())
        .set_attr("class", "min-h-20 w-full")
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
            .init(AGENT_NEW_SESSION_BUTTON.into(), "Start A New Session".into())
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1 join-item",
            )
            // .set_action(TnActionExecutionMethod::Await, query_with_hits)
            .build();

       
        // let rt = tokio::runtime::Runtime::new().unwrap();
        // let chat_textarea_html =
        //     rt.block_on(async { chat_textarea.initial_render().await });
        // //let agent_chat_textarea_string = agent_chat_textarea.initial_render();
        // let query_text_input_html =
        //     rt.block_on(async { query_text_input.initial_render().await });
        // let query_button_html =
        //     rt.block_on(async { query_button.initial_render().await });
        // let new_session_button_html =
        //     rt.block_on(async { new_session_button.initial_render().await });

        // self.html = AgentWorkspaceTemplate {
        //     chat_textarea: chat_textarea_html,
        //     query_text_input: query_text_input_html,
        //     query_button: query_button_html,
        //     new_session_button: new_session_button_html
        // }
        // .render()
        // .unwrap();

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
        let chat_textarea_html = comp_guard
            .get(AGENT_CHAT_TEXTAREA)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

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

fn query(context: TnContext, event: TnEvent, _payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        if event.e_trigger != AGENT_QUERY_BUTTON {
            return None;
        };

        let query_text = context.get_value_from_component(AGENT_QUERY_TEXT_INPUT).await;

        if let TnComponentValue::String(s) =  query_text {
                let query_result_area = context.get_component(AGENT_CHAT_TEXTAREA).await;
                let query = s.replace('\n', "<br>");
    
                chatbox::append_chatbox_value(query_result_area.clone(), ("user".into(), query)).await;
        }

        context.set_ready_for(AGENT_CHAT_TEXTAREA).await;
    

        None
    }
}