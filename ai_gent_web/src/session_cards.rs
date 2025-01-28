use askama::Template;
use async_trait::async_trait;
use chrono::DateTime;
use sqlx::query;
use sqlx::Acquire;
use sqlx::Postgres;
use std::collections::HashMap;
use std::sync::Arc;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use super::DB_POOL;
use chrono::Utc;

/// Represents a button component in a Tron application.
#[non_exhaustive]
#[derive(ComponentBase)]
pub struct SessionCards<'a: 'static> {
    base: TnComponentBase<'a>,
    db_pool: Option<PgPool>,
    username: String,
    status_to_render: String,
    since_then: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct UserData {
    username: String,
    email: String,
}

impl<'a: 'static> SessionCardsBuilder<'a> {
    pub fn init(mut self, tnid: String, title: String, status: &str) -> Self {
        let component_type = TnComponentType::UserDefined("div".into());
        self.base = TnComponentBase::builder(self.base)
            .init("div".into(), tnid, component_type)
            .set_value(TnComponentValue::String(title))
            .build();

        self.status_to_render = status.into();

        self
    }
}

impl<'a: 'static> SessionCards<'a> {
    pub async fn init_db_pool(&mut self) {
        self.db_pool = Some(
          DB_POOL.clone()
        );
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "sessions.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AgentLibraryTemplate {
    cards: Vec<(i32, String, String)>,
}

#[async_trait]
impl<'a> TnComponentRenderTrait<'a> for SessionCards<'a>
where
    'a: 'static,
{
    /// Generates the internal HTML representation of the button component.
    async fn render(&self) -> String {
        let since_then = if let Some(since_then) = self.since_then {
            since_then 
        } else {
            Utc::now() - chrono::Duration::days(3650)
        };
        let pool = self.db_pool.as_ref().expect("Database connection not initialized");
        let chats = query!(
            "SELECT c.chat_id, c.title, c.summary, c.updated_at, a.name AS agent_name, u.username AS username
             FROM chats c
             JOIN agents a ON c.agent_id = a.agent_id
             JOIN users u ON c.user_id = u.user_id
             WHERE u.username = $1 AND c.updated_at > $2 AND c.status = $3 AND a.status = $4 
             ORDER BY c.updated_at DESC",
            self.username,
            since_then,
            "active",
            "active"
        )
        .fetch_all(pool)
        .await
        .unwrap();


        let cards = chats
            .iter()
            .map(|row| {
                let id: i32 = row.chat_id;
                let name: String = row.agent_name.clone();
                let description: String = row.summary.clone().unwrap_or_default();
                (id, name, description)
            })
            .collect::<Vec<_>>();
        
        let html = AgentLibraryTemplate { cards };
        html.render().unwrap()
    }

    /// Generates the initial HTML representation of the button component.
    async fn initial_render(&self) -> String {
        let tron_id = self.tron_id();
        let attributes = HtmlAttributes::builder()
            .add("id", tron_id)
            .add("hx-post", &format!("/tron/{}", tron_id))
            .add("hx-target", &format!("#{}", tron_id))
            .add("hx-swap", "outerHTML")
            .add("hx-trigger", "load")
            .add("hx-ext", "json-enc")
            .add("state", "ready")
            .build()
            .to_string();
        format!("<div {}/>", attributes)
    }

    async fn pre_render(&mut self, ctx: &TnContextBase) {
        if self.db_pool.as_mut().is_none() {
            self.init_db_pool().await;
        }
        //println!("userdata {:?}", ctx.user_data);
        let user_data = ctx.user_data.read().await;
        let json_str = user_data.as_ref().unwrap().clone();
        let user_data: UserData = serde_json::from_str(&json_str).unwrap();
        self.username = user_data.username;
        let assets_guard = ctx.assets.read().await;
        if let Some(TnAsset::U32(since_then_in_days)) = assets_guard.get("since_then_in_days") {
            self.since_then =  Some(Utc::now() - chrono::Duration::days(*since_then_in_days as i64));
        } 
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}
