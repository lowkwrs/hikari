use crate::interaction::CommandCtx;
use hikari_database::repository::antinuke_repository::AntinukeRepository;
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "settings", desc = "View the current AntiNuke configuration")]
pub struct AntinukeSettings {}

pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());

    let (config, modules, wl_count) = tokio::try_join!(
        repo.get_config(cx.guild_id),
        repo.get_module_configs(cx.guild_id),
        repo.get_whitelist_count(cx.guild_id)
    )?;

    let Some(c) = config else {
        return Ok(format!(
            "**AntiNuke Settings**\nStatus: Disabled\nWhitelisted Users: {}\n\nRun `/antinuke enable` to get started.",
            wl_count
        ));
    };

    let log_ch = c
        .log_channel_id
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| "None".into());

    let limits = if modules.is_empty() {
        "No limits configured.".into()
    } else {
        modules
            .iter()
            .map(|m| {
                format!(
                    "- `{}`: {}/{}s ({})",
                    m.action_type, m.threshold, m.window_secs, m.punishment
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Ok(format!(
        "**AntiNuke Settings**\nStatus: {}\nLog Channel: {}\nWhitelisted: {}\n\n**Limits:**\n{}",
        if c.enabled { "On" } else { "Off" },
        log_ch,
        wl_count,
        limits,
    ))
}
