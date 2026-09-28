use crate::core::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "revoke",
    desc = "Revoke premium (Owner: specify user/guild ID)"
)]
pub struct PremiumRevoke {
    #[command(desc = "User ID or Guild ID to revoke")]
    pub target: Option<String>,
}

pub async fn execute(
    cx: &CommandCtx<'_>,
    target_str: Option<&String>,
) -> Result<String, HikariError> {
    let is_owner = hikari_premium::core::engine::PremiumEngine::is_owner(cx.author_id as u64);

    if let Some(target) = target_str {
        if !is_owner {
            return Ok("only bot owner can revoke premium.".to_string());
        }

        if let Ok(target_id) = target.parse::<i64>() {
            // A simple heuristic for discord IDs (guild vs user)
            if target_id > 100000000000000000 {
                hikari_premium::engine()
                    .revoke_guild(target_id)
                    .await
                    .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

                return Ok(format!("premium revoked for guild: {}", target_id));
            } else {
                hikari_premium::engine()
                    .revoke_user(target_id)
                    .await
                    .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

                return Ok(format!("premium revoked for user: {}.", target_id));
            }
        }
    }

    hikari_premium::engine()
        .revoke_guild(cx.guild_id)
        .await
        .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

    Ok("premium revoked for this server.".to_string())
}
