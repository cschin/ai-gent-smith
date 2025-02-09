use askama::Template;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Local;
use sqlx::query;
use sqlx::Acquire;
use sqlx::Postgres;
use std::collections::HashMap;
use std::sync::Arc;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;
use tron_app::HtmlAttributes;

use crate::MOCK_USER;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::{Column, Row, TypeInfo, ValueRef};

use super::DB_POOL;
use chrono::Utc;

#[non_exhaustive]
#[derive(ComponentBase)]
pub struct SessionCards<'a: 'static> {
    base: TnComponentBase<'a>,
    db_pool: Option<PgPool>,
    username: String,
    status_to_render: String,
    since_then: Option<DateTime<Utc>>,
    session_title: String,
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
        self.db_pool = Some(DB_POOL.clone());
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "sessions.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct ChatSessionTemplate {
    cards: Vec<(i32, String, String, String)>,
    session_title: String,
}

#[async_trait]
impl<'a> TnComponentRenderTrait<'a> for SessionCards<'a>
where
    'a: 'static,
{
    async fn render(&self) -> String {
        let since_then = if let Some(since_then) = self.since_then {
            since_then
        } else {
            Utc::now() - chrono::Duration::days(3650)
        };
        let pool = self
            .db_pool
            .as_ref()
            .expect("Database connection not initialized");
        let chats = query!(
            "SELECT c.chat_id, c.title, c.summary, c.updated_at, a.name AS agent_name, u.username AS username, COUNT(m.message_id) AS m_count 
             FROM chats c
             JOIN agents a ON c.agent_id = a.agent_id
             JOIN users u ON c.user_id = u.user_id
             LEFT JOIN messages m on m.chat_id = c.chat_id
             WHERE u.username = $1 AND c.updated_at > $2 AND c.status = $3 AND a.status = $4 
             GROUP BY c.chat_id, c.title, c.summary, c.updated_at, a.name, u.username
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
            .flat_map(|row| {
                if let Some(m_count) = row.m_count {
                    if m_count == 0 {
                        None
                    } else {
                        let id: i32 = row.chat_id;
                        let name: String = ammonia::clean_text(&row.agent_name).to_string();
                        let when: String = if let Some(utc_dt) = row.updated_at {
                            let local_dt = utc_dt.with_timezone(&Local); // Convert to local timezone
                            let formatted_time = local_dt.format("%b %d %H:%M").to_string();
                            formatted_time
                        } else {
                            "".into()
                        };
                        let description: String =
                            ammonia::clean_text(&row.summary.clone().unwrap_or_default())
                                .to_string();
                        Some((id, name, when, description))
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let html = ChatSessionTemplate {
            cards,
            session_title: self.session_title.clone(),
        };
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
        let user_data = ctx.get_user_data().await.unwrap_or(MOCK_USER.clone());
        self.username = user_data.username;

        let assets_guard = ctx.assets.read().await;
        if let Some(TnAsset::U32(since_then_in_days)) = assets_guard.get("since_then_in_days") {
            self.since_then = Some(Utc::now() - chrono::Duration::days(*since_then_in_days as i64));
        }
        if let Some(TnAsset::String(session_title)) = assets_guard.get("session_title") {
            self.session_title = session_title.to_string();
        }
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}
