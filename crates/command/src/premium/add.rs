use crate::core::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::id::{Id, marker::UserMarker};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "add", desc = "Add premium to a user (Owner only)")]
pub struct PremiumAdd {
    #[command(desc = "The user to grant premium")]
    pub user: Id<UserMarker>,
}

pub async fn execute(cx: &CommandCtx<'_>, target_id: i64) -> Result<String, HikariError> {
    if !hikari_premium::core::engine::PremiumEngine::is_owner(cx.author_id as u64) {
        return Ok("only bot owner can add premium.".to_string());
    }

    hikari_premium::engine()
        .add_user(target_id, cx.author_id)
        .await
        .map_err(|e| HikariError::Internal(format!("DB error: {}", e)))?;

    Ok(format!("granted premium to <@{}>.", target_id))
}
