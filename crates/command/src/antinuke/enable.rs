use crate::interaction::CommandCtx;
use chrono::Utc;
use hikari_database::{
    models::antinuke_config::AntinukeConfig, repository::antinuke_repository::AntinukeRepository,
};
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
    id::Id,
};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "enable", desc = "Enable AntiNuke protection for this server")]
pub struct AntinukeEnable {}

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

    if config.enabled {
        return Ok("antinuke is already on.".into());
    }

    if config.log_channel_id.is_none() {
        let channel = cx
            .module
            .discord
            .create_guild_channel(Id::new(cx.guild_id as u64), "hikari-logs")
            .permission_overwrites(&[PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                id: Id::new(cx.guild_id as u64),
                kind: PermissionOverwriteType::Role,
            }])
            .await?
            .model()
            .await?;
        config.log_channel_id = Some(channel.id.get() as i64);
    }

    config.enabled = true;
    repo.upsert_config(&config).await?;
    hikari_antinuke::reload_guild_config(&cx.module.db, cx.guild_id as u64).await;

    Ok("antinuke is now **on**. logs will be sent to `#hikari-logs`.".into())
}
