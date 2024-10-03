use askama::Template;
use axum::async_trait;
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

/// Represents a button component in a Tron application.
#[non_exhaustive]
#[derive(ComponentBase)]
pub struct LibraryCards<'a: 'static> {
    base: TnComponentBase<'a>,
    db_pool: Option<PgPool>,
    user_data: String,
    status_to_render: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct UserData {
    username: String,
    email: String,
}

impl<'a: 'static> LibraryCardsBuilder<'a> {
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

impl<'a: 'static> LibraryCards<'a> {
    pub async fn init_db_pool(&mut self) {
        self.db_pool = Some(
          DB_POOL.clone()
        );
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "agent_library.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
struct AgentLibraryTemplate {
    cards: Vec<(i32, String, String)>,
}

#[async_trait]
impl<'a> TnComponentRenderTrait<'a> for LibraryCards<'a>
where
    'a: 'static,
{
    /// Generates the internal HTML representation of the button component.
    async fn render(&self) -> String {
        let query = format!(
            "SELECT a.agent_id, a.name, a.description
            FROM agents a
            JOIN users u ON a.user_id = u.user_id
            WHERE u.username = '{}' AND a.status = '{}' ORDER BY a.agent_id ASC;",
            self.user_data,
            self.status_to_render
        );
        let pool = self.db_pool.as_ref().expect("Database connection not initialized");
        let rows = sqlx::query(&query).fetch_all(pool).await.expect("db error");

        let cards = rows
            .iter()
            .map(|row| {
                let id: i32 = row.try_get("agent_id").unwrap_or_default();
                let name: String = row.try_get("name").unwrap_or_default();
                let description: String = row.try_get("description").unwrap_or_default();
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
        self.user_data = user_data.username;
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}
