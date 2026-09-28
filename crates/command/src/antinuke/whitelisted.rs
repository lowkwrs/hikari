use crate::interaction::CommandCtx;
use hikari_database::repository::antinuke_repository::AntinukeRepository;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "whitelisted",
    desc = "View all users whitelisted from AntiNuke"
)]
pub struct AntinukeWhitelisted {}

pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());
    let list = repo.get_whitelist(cx.guild_id).await.unwrap_or_default();
    let count = list.len();

    if count == 0 {
        return Ok("**0** users whitelisted from antinuke.".into());
    }

    let joined = list
        .iter()
        .map(|id| format!("<@{}>", id))
        .collect::<Vec<_>>()
        .join(", ");

    let body = if joined.len() > 1800 {
        format!("{}… and more", &joined[..1800])
    } else {
        joined
    };

    Ok(format!("**{}** whitelisted user(s):\n{}", count, body))
}
