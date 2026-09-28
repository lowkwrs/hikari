use crate::core::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "noprefix", desc = "Toggle no-prefix feature")]
pub struct PremiumNoprefix {}

pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let is_guild_premium = hikari_premium::engine().is_guild_premium(cx.guild_id).await;

    if !is_guild_premium {
        return Ok("server does not have premium.".to_string());
    }

    let current = hikari_premium::engine()
        .is_noprefix_enabled(cx.guild_id)
        .await;
    hikari_premium::engine()
        .toggle_noprefix(cx.guild_id, !current)
        .await
        .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

    let desc = if !current {
        "noprefix enabled."
    } else {
        "noprefix disabled."
    };

    Ok(desc.to_string())
}
