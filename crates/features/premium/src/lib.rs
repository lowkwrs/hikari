pub mod core;

use hikari_database::pool::Database;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use hikari_utils::module::Module;
use std::sync::OnceLock;

static ENGINE: OnceLock<core::engine::PremiumEngine> = OnceLock::new();

pub fn engine() -> &'static core::engine::PremiumEngine {
    ENGINE.get().expect("PremiumEngine not initialized")
}

pub struct PremiumModule {
    _db: Database,
}

impl PremiumModule {
    pub fn new(db: Database) -> Self {
        ENGINE.get_or_init(|| core::engine::PremiumEngine::new(db.clone()));
        Self { _db: db }
    }
}

impl Module for PremiumModule {
    fn name(&self) -> &'static str {
        "PremiumModule"
    }

    async fn handle_event(
        &self,
        _event: &HikariEvent,
        _ctx: &hikari_utils::module::ModuleContext,
    ) -> Result<(), HikariError> {
        Ok(())
    }
}
