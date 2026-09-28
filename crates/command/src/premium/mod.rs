pub mod activate;
pub mod add;
pub mod noprefix;
pub mod revoke;
pub mod status;

use crate::core::handler::ModuleCommand;
use crate::{
    core::interaction::{CommandCtx, InteractionContext},
    core::prefix::PrefixContext,
};
use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::InteractionData;

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "premium", desc = "Manage premium features")]
pub enum PremiumCommand {
    #[command(name = "activate")]
    Activate(activate::PremiumActivate),
    #[command(name = "add")]
    Add(add::PremiumAdd),
    #[command(name = "noprefix")]
    NoPrefix(noprefix::PremiumNoprefix),
    #[command(name = "revoke")]
    Revoke(revoke::PremiumRevoke),
    #[command(name = "status")]
    Status(status::PremiumStatus),
}

impl PremiumCommand {
    pub async fn run(&self, cx: &CommandCtx<'_>) -> Result<String, HikariError> {
        match self {
            Self::Activate(_) => activate::execute(cx).await,
            Self::Add(a) => add::execute(cx, a.user.get() as i64).await,
            Self::NoPrefix(_) => noprefix::execute(cx).await,
            Self::Revoke(r) => revoke::execute(cx, r.target.as_ref()).await,
            Self::Status(_) => status::execute(cx).await,
        }
    }
}

pub async fn handle_slash(
    interaction_ctx: InteractionContext,
    module: &ModuleContext,
) -> Result<(), HikariError> {
    let guild_id = interaction_ctx
        .guild_id()
        .ok_or_else(|| HikariError::Internal("Guild only".into()))?;
    let user_id = interaction_ctx
        .author_id()
        .ok_or_else(|| HikariError::Internal("No author".into()))?;

    let cmd = match &interaction_ctx.interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => {
            PremiumCommand::from_interaction((**data).clone().into())
                .map_err(|e| HikariError::Internal(format!("Parse error: {e}")))?
        }
        _ => return Err(HikariError::Internal("Not a command".into())),
    };

    interaction_ctx.defer(module).await?;

    let cx = CommandCtx::new(guild_id.get() as i64, user_id.get() as i64, module);
    let reply = cmd.run(&cx).await?;

    let embed = hikari_embeds::ui::build_stylish_embed("Premium", &reply, module.embed_color);

    interaction_ctx.respond_embed(module, embed, None).await
}

pub struct PremiumHandler;

impl Default for PremiumHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PremiumHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ModuleCommand for PremiumHandler {
    fn name(&self) -> &'static str {
        "premium"
    }

    fn command(&self) -> twilight_model::application::command::Command {
        PremiumCommand::create_command().into()
    }

    async fn handle_slash(
        &self,
        ctx: InteractionContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        handle_slash(ctx, module).await
    }

    async fn handle_prefix(
        &self,
        _ctx: &PrefixContext,
        _module: &ModuleContext,
    ) -> Result<(), HikariError> {
        Ok(())
    }

    async fn handle_component(
        &self,
        _interaction_ctx: &InteractionContext,
        _custom_id: &str,
        _module: &ModuleContext,
    ) -> Result<bool, HikariError> {
        Ok(false)
    }
}
