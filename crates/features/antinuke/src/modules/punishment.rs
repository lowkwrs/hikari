use crate::core::engine::AntiNukeEngine;
use crate::models::types::{Punishment, ThreatResult};
use hikari_database::pool::Database;
use hikari_utils::error::HikariError;
use std::sync::Arc;
use tracing::debug;
use twilight_http::Client as HttpClient;
use twilight_http::request::AuditLogReason;
use twilight_model::id::{
    Id,
    marker::{GuildMarker, RoleMarker, UserMarker},
};

pub async fn execute(
    engine: &AntiNukeEngine,
    http: &Arc<HttpClient>,
    db: &Database,
    guild_id: u64,
    user_id: u64,
    result: &ThreatResult,
    redis_conn: &mut deadpool_redis::Connection,
) -> Result<(), HikariError> {
    let t_guild_id = Id::<GuildMarker>::new(guild_id);
    let t_user_id = Id::<UserMarker>::new(user_id);

    match result.punishment {
        Punishment::Ban => {
            if let Err(e) = http
                .create_ban(t_guild_id, t_user_id)
                .delete_message_seconds(0)
                .reason(&result.reason)
                .await
            {
                tracing::warn!("[ANTINUKE] Failed to ban {}: {}", user_id, e);
            } else {
                debug!(
                    "[ANTINUKE] BANNED user {} in guild {} — {}",
                    user_id, guild_id, result.reason
                );
            }
        }
        Punishment::Kick => {
            if let Err(e) = http
                .remove_guild_member(t_guild_id, t_user_id)
                .reason(&result.reason)
                .await
            {
                tracing::warn!("[ANTINUKE] Failed to kick {}: {}", user_id, e);
            } else {
                debug!(
                    "[ANTINUKE] KICKED user {} in guild {} — {}",
                    user_id, guild_id, result.reason
                );
            }
        }
        Punishment::Timeout => {
            if let Ok(until) = twilight_model::util::datetime::Timestamp::from_secs(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
                    + (30 * 60),
            ) {
                let _ = http
                    .update_guild_member(t_guild_id, t_user_id)
                    .communication_disabled_until(Some(until))
                    .reason(&result.reason)
                    .await;
                debug!(
                    "[ANTINUKE] TIMED OUT user {} in guild {}",
                    user_id, guild_id
                );
            }
        }
        Punishment::StripRoles => {
            if let Some(cached_roles) = engine.get_member_roles(guild_id, user_id).await {
                let mut keep = Vec::new();
                for &rid in &cached_roles {
                    let perms = engine.get_role_perms(guild_id, rid).await;
                    if !super::scorer::has_dangerous_perm_grant(0, perms) {
                        keep.push(Id::<RoleMarker>::new(rid));
                    }
                }
                if let Err(e) = http
                    .update_guild_member(t_guild_id, t_user_id)
                    .roles(&keep)
                    .reason(&result.reason)
                    .await
                {
                    tracing::warn!("[ANTINUKE] Failed to strip roles from {}: {}", user_id, e);
                } else {
                    debug!(
                        "[ANTINUKE] STRIPPED ROLES from user {} in guild {}",
                        user_id, guild_id
                    );
                }
            } else if let Ok(resp) = http.guild_member(t_guild_id, t_user_id).await
                && let Ok(member) = resp.model().await
                && let Ok(resp) = http.roles(t_guild_id).await
                && let Ok(roles) = resp.model().await
            {
                let keep: Vec<_> = member
                    .roles
                    .iter()
                    .filter(|&&rid| {
                        roles
                            .iter()
                            .find(|r| r.id == rid)
                            .map(|r| {
                                !super::scorer::has_dangerous_perm_grant(0, r.permissions.bits())
                            })
                            .unwrap_or(true)
                    })
                    .copied()
                    .collect();
                let _ = http
                    .update_guild_member(t_guild_id, t_user_id)
                    .roles(&keep)
                    .reason(&result.reason)
                    .await;
            }
            debug!(
                "[ANTINUKE] STRIPPED ROLES from user {} in guild {} (fallback)",
                user_id, guild_id
            );
        }
        Punishment::LogOnly | Punishment::None => {}
    }

    engine.clear_user(guild_id, user_id, redis_conn).await;

    let log_channel_id = engine.get_log_channel(guild_id).await;

    let http = Arc::clone(http);
    let db_pool = db.pool.clone();
    let result_clone = result.clone();

    tokio::spawn(async move {
        let action_str = result_clone.action.as_str();
        let punishment_str = result_clone.punishment.as_str();
        let score = result_clone.score as i32;
        let count = result_clone.count_in_window as i32;

        let db_fut = tokio::spawn(async move {
            let _ = sqlx::query!(
                r#"
                INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                guild_id as i64,
                user_id as i64,
                action_str,
                score,
                punishment_str,
                count
            )
            .execute(&db_pool)
            .await;
        });

        let http2 = Arc::clone(&http);
        let discord_fut = tokio::spawn(async move {
            if let Some(log_ch_id) = log_channel_id {
                let color = if result_clone.triggered {
                    0xFF0000_u32
                } else {
                    0x00FF00_u32
                };

                let timestamp = twilight_model::util::datetime::Timestamp::from_secs(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                )
                .ok();

                let mut embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title(if result_clone.triggered {
                        "AntiNuke Triggered"
                    } else {
                        "Action Reverted"
                    })
                    .color(color)
                    .description(&result_clone.reason)
                    .field(
                        twilight_util::builder::embed::EmbedFieldBuilder::new(
                            "Action",
                            result_clone.action.as_str(),
                        )
                        .inline(),
                    )
                    .field(
                        twilight_util::builder::embed::EmbedFieldBuilder::new(
                            "Punishment",
                            result_clone.punishment.as_str(),
                        )
                        .inline(),
                    )
                    .field(
                        twilight_util::builder::embed::EmbedFieldBuilder::new(
                            "User",
                            format!("<@{}>", user_id),
                        )
                        .inline(),
                    )
                    .field(
                        twilight_util::builder::embed::EmbedFieldBuilder::new(
                            "Score",
                            format!("{}/100", result_clone.score),
                        )
                        .inline(),
                    );

                if let Some(ts) = timestamp {
                    embed = embed.timestamp(ts);
                }

                let built_embed = embed.build();
                let embeds = [built_embed];
                if result_clone.triggered {
                    let buttons = hikari_embeds::ui::build_antinuke_log_buttons(user_id);
                    let components = [buttons];
                    let _ = http2
                        .create_message(Id::new(log_ch_id))
                        .embeds(&embeds)
                        .components(&components)
                        .await;
                } else {
                    let _ = http2
                        .create_message(Id::new(log_ch_id))
                        .embeds(&embeds)
                        .await;
                }
            }
        });

        let _ = tokio::join!(db_fut, discord_fut);
    });

    Ok(())
}
