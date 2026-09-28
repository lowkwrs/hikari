use hikari_antinuke::AntinukeModule;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use hikari_utils::module::{Module, ModuleContext};

use hikari_database::pool::Database;
use std::sync::Arc;
use twilight_http::Client as HttpClient;

pub struct ModuleRegistry {
    antinuke: AntinukeModule,
}

impl ModuleRegistry {
    pub fn new(http: Arc<HttpClient>, db: Database, redis_pool: hikari_cache::RedisPool) -> Self {
        Self {
            antinuke: AntinukeModule::new(http, db, redis_pool),
        }
    }

    pub async fn handle_event(
        &self,
        event: &HikariEvent,
        ctx: &ModuleContext,
    ) -> Result<(), HikariError> {
        // Sequentially route the event to each registered module
        self.antinuke.handle_event(event, ctx).await?;

        Ok(())
    }
}
