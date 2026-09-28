use crate::core::interaction::CommandCtx;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "status", desc = "Check premium status")]
pub struct PremiumStatus {}

pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let is_owner = hikari_premium::core::engine::PremiumEngine::is_owner(cx.author_id as u64);
    let is_user_premium = hikari_premium::engine().is_user_premium(cx.author_id).await;
    let is_guild_premium = hikari_premium::engine().is_guild_premium(cx.guild_id).await;
    let noprefix = hikari_premium::engine()
        .is_noprefix_enabled(cx.guild_id)
        .await;

    let mut desc = String::new();

    if is_owner {
        desc.push_str("**Account:** Owner\n");
    } else if is_user_premium {
        desc.push_str("**Account:** Premium\n");
    } else {
        desc.push_str("**Account:** Free\n");
    }

    desc.push_str(&format!(
        "**Server Premium:** {}\n**No Prefix:** {}",
        if is_guild_premium { "Yes" } else { "No" },
        if noprefix { "On" } else { "Off" }
    ));

    Ok(desc)
}
