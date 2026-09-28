use crate::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "enable",
    desc = "Enable AutoMod protection rules for this server"
)]
pub struct AutomodEnable {}

#[tracing::instrument(skip_all, fields(guild = cx.guild_id))]
pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let count = hikari_automod::enable_rules(cx.guild_id as u64, &cx.module.discord).await;
    Ok(format!("automod is now **on**. ({count} rules active)"))
}
