use crate::interaction::CommandCtx;
use hikari_database::repository::antinuke_repository::AntinukeRepository;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::id::{Id, marker::UserMarker};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "add", desc = "Add a user to the AntiNuke whitelist")]
pub struct WhitelistAdd {
    #[command(desc = "The user to exempt from AntiNuke")]
    pub user: Id<UserMarker>,
}

pub async fn execute(cx: &CommandCtx<'_>, user_id: i64) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());
    repo.add_whitelist(cx.guild_id, user_id, cx.author_id)
        .await?;
    hikari_antinuke::whitelist_add(cx.guild_id as u64, user_id as u64).await;
    Ok(format!("<@{user_id}> added to the antinuke whitelist."))
}
