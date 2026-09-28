use crate::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "disable",
    desc = "Remove all hikari AutoMod rules from this server"
)]
pub struct AutomodDisable {}

#[tracing::instrument(skip_all, fields(guild = cx.guild_id))]
pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let count = hikari_automod::disable_rules(cx.guild_id as u64, &cx.module.discord).await;
    Ok(format!("automod is now **off**. ({count} rules removed)"))
}
