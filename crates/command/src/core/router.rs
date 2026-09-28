use crate::core::handler::ModuleCommand;
use crate::interaction::InteractionContext;
use crate::prefix::PrefixRouter;
use hikari_utils::{error::HikariError, module::ModuleContext};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct CommandRouter {
    prefix_router: PrefixRouter,
    commands: HashMap<&'static str, Arc<dyn ModuleCommand>>,
    prefix_lookup: HashMap<&'static str, Arc<dyn ModuleCommand>>,
    builtin_names: HashSet<&'static str>,
}

impl CommandRouter {
    pub fn new(prefix: String) -> Self {
        let mut builtin_names = HashSet::new();
        builtin_names.insert("ping");

        Self {
            prefix_router: PrefixRouter::new(prefix),
            commands: HashMap::new(),
            prefix_lookup: HashMap::new(),
            builtin_names,
        }
    }

    pub fn register(&mut self, cmd: Arc<dyn ModuleCommand>) {
        let name = cmd.name();
        self.prefix_lookup.insert(name, cmd.clone());
        for alias in cmd.prefix_aliases() {
            self.prefix_lookup.insert(alias, cmd.clone());
        }
        self.commands.insert(name, cmd);
    }

    pub async fn handle_event(
        &self,
        event: &hikari_utils::event::HikariEvent,
        ctx: &ModuleContext,
    ) -> Result<(), HikariError> {
        if let hikari_utils::event::HikariEvent::Discord(box_event) = event {
            match &**box_event {
                twilight_model::gateway::event::Event::InteractionCreate(interaction) => {
                    let interaction_ctx = InteractionContext::new(interaction.0.clone());
                    return self.route(interaction_ctx, ctx).await;
                }
                twilight_model::gateway::event::Event::MessageCreate(msg) => {
                    return self
                        .prefix_router
                        .handle_message(&msg.0, ctx, &self.prefix_lookup, &self.builtin_names)
                        .await;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn route(
        &self,
        interaction_ctx: InteractionContext,
        ctx: &ModuleContext,
    ) -> Result<(), HikariError> {
        if interaction_ctx.is_component() {
            let custom_id = interaction_ctx.extract_custom_id()?.to_string();
            return self
                .handle_component(&interaction_ctx, &custom_id, ctx)
                .await;
        }

        let name = interaction_ctx.extract_command_name()?.to_string();
        if let Some(cmd) = self.commands.get(name.as_str()) {
            cmd.handle_slash(interaction_ctx, ctx).await?;
        } else {
            tracing::warn!("Unknown command: {}", name);
        }
        Ok(())
    }

    async fn handle_component(
        &self,
        interaction_ctx: &InteractionContext,
        custom_id: &str,
        ctx: &ModuleContext,
    ) -> Result<(), HikariError> {
        for cmd in self.commands.values() {
            if cmd
                .handle_component(interaction_ctx, custom_id, ctx)
                .await?
            {
                return Ok(());
            }
        }
        tracing::warn!("Unhandled component: {}", custom_id);
        Ok(())
    }
}
