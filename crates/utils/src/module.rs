use crate::error::HikariError;
use crate::event::HikariEvent;
use sqlx::PgPool;
use std::sync::Arc;
use twilight_http::Client as DiscordClient;

pub struct ModuleContext {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub discord: Arc<DiscordClient>,
    pub embed_color: u32,
}

pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;

    fn handle_event(
        &self,
        event: &HikariEvent,
        ctx: &ModuleContext,
    ) -> impl std::future::Future<Output = Result<(), HikariError>> + Send;
}
