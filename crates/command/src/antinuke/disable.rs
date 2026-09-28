use crate::interaction::CommandCtx;
use chrono::Utc;
use hikari_database::{
    models::antinuke_config::AntinukeConfig, repository::antinuke_repository::AntinukeRepository,
};
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::id::Id;

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "disable", desc = "Disable AntiNuke protection for this server")]
pub struct AntinukeDisable {}

#[tracing::instrument(skip_all, fields(guild = cx.guild_id))]
pub async fn execute(cx: &CommandCtx<'_>) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());
    let mut config = repo
        .get_config(cx.guild_id)
        .await?
        .unwrap_or_else(|| AntinukeConfig {
            guild_id: cx.guild_id,
            enabled: false,
            log_channel_id: None,
            updated_at: Utc::now(),
        });

    if !config.enabled {
        return Ok("antinuke is already **off**.".into());
    }

    if let Some(ch_id) = config.log_channel_id {
        let _ = cx
            .module
            .discord
            .delete_channel(Id::new(ch_id as u64))
            .await;
        config.log_channel_id = None;
    }

    config.enabled = false;
    repo.upsert_config(&config).await?;
    hikari_antinuke::reload_guild_config(&cx.module.db, cx.guild_id as u64).await;

    Ok("antinuke is now **off**. log channel removed.".into())
}
