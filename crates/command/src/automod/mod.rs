pub mod disable;
pub mod enable;

use crate::core::handler::ModuleCommand;
use crate::{
    interaction::{CommandCtx, InteractionContext},
    prefix::PrefixContext,
};
use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::InteractionData;

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "automod", desc = "Configure the hikari AutoMod engine")]
pub enum AutomodCommand {
    #[command(name = "enable")]
    Enable(enable::AutomodEnable),
    #[command(name = "disable")]
    Disable(disable::AutomodDisable),
}

impl AutomodCommand {
    pub async fn run(&self, cx: &CommandCtx<'_>) -> Result<String, HikariError> {
        match self {
            Self::Enable(_) => enable::execute(cx).await,
            Self::Disable(_) => disable::execute(cx).await,
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

    interaction_ctx.defer(module).await?;

    let cmd = match &interaction_ctx.interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => {
            AutomodCommand::from_interaction((**data).clone().into())
                .map_err(|e| HikariError::Internal(format!("Parse error: {e}")))?
        }
        _ => return Err(HikariError::Internal("Not a command".into())),
    };

    let cx = CommandCtx::new(guild_id.get() as i64, 0, module);
    let reply = cmd.run(&cx).await?;
    let embed = hikari_embeds::ui::build_stylish_embed("AutoMod", &reply, module.embed_color);

    interaction_ctx.respond_embed(module, embed, None).await
}

pub async fn handle_prefix(
    prefix_ctx: &PrefixContext,
    module: &ModuleContext,
) -> Result<(), HikariError> {
    let guild_id = prefix_ctx.guild_id.get() as i64;

    let cmd = match prefix_ctx
        .args
        .first()
        .map(|s: &String| s.to_lowercase())
        .as_deref()
    {
        Some("enable") => AutomodCommand::Enable(enable::AutomodEnable {}),
        Some("disable") => AutomodCommand::Disable(disable::AutomodDisable {}),
        _ => {
            let embed = hikari_embeds::ui::build_stylish_embed(
                "AutoMod Help",
                "Commands: `enable` · `disable`",
                module.embed_color,
            );
            return prefix_ctx.reply_with_ui(embed, vec![]).await;
        }
    };

    let cx = CommandCtx::new(guild_id, 0, module);
    let reply = cmd.run(&cx).await?;
    let embed = hikari_embeds::ui::build_stylish_embed("AutoMod", &reply, module.embed_color);
    prefix_ctx.reply_with_ui(embed, vec![]).await
}

pub struct AutomodHandler;

#[async_trait]
impl ModuleCommand for AutomodHandler {
    fn name(&self) -> &'static str {
        "automod"
    }

    fn command(&self) -> twilight_model::application::command::Command {
        AutomodCommand::create_command().into()
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
        ctx: &PrefixContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        handle_prefix(ctx, module).await
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
