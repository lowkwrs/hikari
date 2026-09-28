use crate::core::handler::ModuleCommand;
use crate::{interaction::InteractionContext, prefix::PrefixContext};
use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "ping", desc = "Check bot latency")]
pub struct PingCommand {}

pub struct PingHandler;

#[async_trait]
impl ModuleCommand for PingHandler {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn command(&self) -> twilight_model::application::command::Command {
        PingCommand::create_command().into()
    }

    async fn handle_slash(
        &self,
        ctx: InteractionContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        let start = std::time::Instant::now();
        let _ = module.discord.current_user().await;
        let elapsed = start.elapsed().as_millis();

        let text = format!("pong! `{}ms`", elapsed);
        let embed = hikari_embeds::ui::build_stylish_embed("Ping", &text, module.embed_color);
        ctx.respond_embed(module, embed, None).await
    }

    async fn handle_prefix(
        &self,
        ctx: &PrefixContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        let start = std::time::Instant::now();
        let _ = module.discord.current_user().await;
        let elapsed = start.elapsed().as_millis();

        let text = format!("pong! `{}ms`", elapsed);
        let embed = hikari_embeds::ui::build_stylish_embed("Ping", &text, module.embed_color);
        ctx.reply_with_ui(embed, vec![]).await
    }

    async fn handle_component(
        &self,
        _ctx: &InteractionContext,
        _custom_id: &str,
        _module: &ModuleContext,
    ) -> Result<bool, HikariError> {
        Ok(false)
    }
}
