use crate::core::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "activate", desc = "Activate premium for this server")]
pub struct PremiumActivate {}

pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let is_premium = hikari_premium::engine().is_user_premium(cx.author_id).await;

    if !is_premium {
        return Ok("you don't have premium.".to_string());
    }

    hikari_premium::engine()
        .activate_guild(cx.guild_id, cx.author_id)
        .await
        .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

    Ok("premium activated for this server.".to_string())
}
