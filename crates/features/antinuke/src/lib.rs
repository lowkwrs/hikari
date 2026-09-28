pub mod core;
pub mod models;
pub mod modules;

pub use core::engine;
pub use core::recovery;
pub use core::snapshot;
pub use core::whitelist;
pub use models::config;
pub use models::types;
pub use modules::action;
pub use modules::content;
pub use modules::punishment;
pub use modules::scorer;

use hikari_cache::RedisPool;
use hikari_database::pool::Database;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use hikari_utils::module::Module;
use std::sync::OnceLock;

use std::sync::Arc;
use tracing::debug;
use twilight_http::Client as HttpClient;
use twilight_model::id::Id;

static ENGINE: OnceLock<engine::AntiNukeEngine> = OnceLock::new();
static RECOVERY: OnceLock<recovery::RecoverySystem> = OnceLock::new();
static OWNER_CACHE: std::sync::LazyLock<dashmap::DashMap<u64, u64>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);

fn engine() -> &'static engine::AntiNukeEngine {
    ENGINE.get().expect("AntiNukeEngine not initialized")
}

fn recovery() -> &'static recovery::RecoverySystem {
    RECOVERY.get().expect("RecoverySystem not initialized")
}

pub async fn reload_guild_config(pool: &sqlx::PgPool, guild_id: u64) {
    let repo =
        hikari_database::repository::antinuke_repository::AntinukeRepository::new(pool.clone());
    let guild_id_i64 = guild_id as i64;

    let db_config = match repo.get_config(guild_id_i64).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let db_modules = match repo.get_module_configs(guild_id_i64).await {
        Ok(m) => m,
        _ => return,
    };

    let mut modules = std::collections::HashMap::new();
    for m in db_modules {
        if let Some(action) = types::ActionType::parse(&m.action_type) {
            let punishment =
                types::Punishment::parse(&m.punishment).unwrap_or(types::Punishment::None);
            modules.insert(
                action as u8,
                types::InternalModuleConfig {
                    enabled: m.enabled,
                    threshold: m.threshold as u32,
                    window_secs: m.window_secs as u32,
                    punishment,
                    log_only: m.log_only,
                },
            );
        }
    }

    let config = types::GuildConfig {
        enabled: db_config.enabled,
        modules,
        log_channel_id: db_config.log_channel_id.map(|id| id as u64),
    };
    let whitelists = repo.get_whitelist(guild_id_i64).await.unwrap_or_default();
    engine()
        .configure(
            guild_id,
            config,
            whitelists.into_iter().map(|id| id as u64).collect(),
        )
        .await;
    debug!(
        "[ANTINUKE] Reloaded config for guild {} into engine",
        guild_id
    );
}

pub async fn whitelist_add(guild_id: u64, user_id: u64) {
    engine().whitelist_add(guild_id, user_id).await;
}

pub async fn whitelist_remove(guild_id: u64, user_id: u64) {
    engine().whitelist_remove(guild_id, user_id).await;
}

#[inline]
fn is_bot(user_id: u64) -> bool {
    let cached = hikari_utils::ids::get_bot_id();
    cached != 0 && cached == user_id
}

#[inline]
fn is_owner(guild_id: u64, user_id: u64) -> bool {
    OWNER_CACHE
        .get(&guild_id)
        .map(|v| *v == user_id)
        .unwrap_or(false)
}

#[inline]
fn should_skip(guild_id: u64, user_id: u64) -> bool {
    is_bot(user_id) || is_owner(guild_id, user_id)
}

pub struct AntinukeModule {
    http: Arc<HttpClient>,
    db: Database,
}

impl AntinukeModule {
    pub fn new(http: Arc<HttpClient>, db: Database, redis_pool: RedisPool) -> Self {
        let engine = engine::AntiNukeEngine::new(redis_pool).with_ban_executor(http.clone());
        ENGINE.get_or_init(|| engine);
        RECOVERY.get_or_init(|| recovery::RecoverySystem::new(http.clone()));
        Self { http, db }
    }
}

impl Module for AntinukeModule {
    fn name(&self) -> &'static str {
        "AntinukeModule"
    }

    async fn handle_event(
        &self,
        event: &HikariEvent,
        _ctx: &hikari_utils::module::ModuleContext,
    ) -> Result<(), HikariError> {
        if let HikariEvent::Discord(box_event) = event {
            match &**box_event {
                twilight_model::gateway::event::Event::GuildCreate(ev) => {
                    if let twilight_model::gateway::payload::incoming::GuildCreate::Available(
                        guild,
                    ) = ev.as_ref()
                    {
                        let guild_id = guild.id.get();

                        OWNER_CACHE.insert(guild_id, guild.owner_id.get());

                        if hikari_utils::ids::get_bot_id() == 0
                            && let Ok(resp) = self.http.current_user().await
                            && let Ok(me) = resp.model().await
                        {
                            hikari_utils::ids::set_bot_id(me.id.get());
                        }

                        reload_guild_config(&self.db.pool, guild_id).await;

                        engine().whitelist_add(guild_id, guild.owner_id.get()).await;
                        if hikari_utils::ids::get_bot_id() != 0 {
                            engine()
                                .whitelist_add(guild_id, hikari_utils::ids::get_bot_id())
                                .await;
                        }

                        let channels: Vec<snapshot::ChannelSnap> = guild
                            .channels
                            .iter()
                            .map(|c| {
                                let overwrites = c.permission_overwrites.as_ref().map(|ov| {
                                    ov.iter().map(|o| snapshot::PermOverwrite {
                                        id: o.id.get(),
                                        kind: match o.kind {
                                            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Role => 0,
                                            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Member => 1,
                                            _ => 0,
                                        },
                                        allow: o.allow.bits(),
                                        deny: o.deny.bits(),
                                    }).collect()
                                }).unwrap_or_default();

                                snapshot::ChannelSnap {
                                    id: c.id.get(),
                                    name: c.name.clone().unwrap_or_default(),
                                    kind: c.kind.into(),
                                    position: c.position.unwrap_or(0),
                                    topic: c.topic.clone(),
                                    nsfw: c.nsfw.unwrap_or(false),
                                    rate_limit_per_user: c.rate_limit_per_user.unwrap_or(0),
                                    parent_id: c.parent_id.map(|pid| pid.get()),
                                    overwrites,
                                }
                            })
                            .collect();
                        let roles: Vec<snapshot::RoleSnap> = guild
                            .roles
                            .iter()
                            .map(|r| snapshot::RoleSnap {
                                id: r.id.get(),
                                name: r.name.clone(),
                                color: r.colors.primary_color,
                                permissions: r.permissions.bits(),
                                position: r.position,
                                hoist: r.hoist,
                                mentionable: r.mentionable,
                            })
                            .collect();
                        engine()
                            .set_snapshot(snapshot::GuildSnapshot {
                                guild_id,
                                name: guild.name.clone(),
                                channels,
                                roles,
                            })
                            .await;

                        for member in &guild.members {
                            engine()
                                .set_member_roles(
                                    guild_id,
                                    member.user.id.get(),
                                    member.roles.iter().map(|r| r.get()).collect(),
                                )
                                .await;
                        }
                    }
                }

                twilight_model::gateway::event::Event::ChannelCreate(ev) => {
                    if let Some(gid) = ev.guild_id {
                        let overwrites = ev.permission_overwrites.as_ref().map(|ov| {
                            ov.iter().map(|o| snapshot::PermOverwrite {
                                id: o.id.get(),
                                kind: match o.kind {
                                    twilight_model::channel::permission_overwrite::PermissionOverwriteType::Role => 0,
                                    twilight_model::channel::permission_overwrite::PermissionOverwriteType::Member => 1,
                                    _ => 0,
                                },
                                allow: o.allow.bits(),
                                deny: o.deny.bits(),
                            }).collect()
                        }).unwrap_or_default();

                        let snap = snapshot::ChannelSnap {
                            id: ev.id.get(),
                            name: ev.name.clone().unwrap_or_default(),
                            kind: ev.kind.into(),
                            position: ev.position.unwrap_or(0),
                            topic: ev.topic.clone(),
                            nsfw: ev.nsfw.unwrap_or(false),
                            rate_limit_per_user: ev.rate_limit_per_user.unwrap_or(0),
                            parent_id: ev.parent_id.map(|id| id.get()),
                            overwrites,
                        };
                        engine().upsert_channel_snap(gid.get(), snap).await;
                    }
                }

                twilight_model::gateway::event::Event::ChannelDelete(ev) => {
                    if let Some(gid) = ev.guild_id {
                        let channel_id = ev.id.get();
                        let guild_id = gid.get();

                        let http = self.http.clone();
                        let db = self.db.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                            let audit_result = tokio::time::timeout(
                                std::time::Duration::from_millis(150),
                                http.audit_log(Id::new(guild_id))
                                    .action_type(twilight_model::guild::audit_log::AuditLogEventType::ChannelDelete)
                                    .limit(1)
                            ).await;

                            let mut should_recover = false;
                            if let Ok(Ok(resp)) = audit_result
                                && let Ok(audit) = resp.model().await
                                && let Some(entry) = audit.entries.first()
                                && let Some(user_id) = entry.user_id
                            {
                                let uid = user_id.get();

                                if should_skip(guild_id, uid) {
                                    return;
                                }

                                if !engine().is_whitelisted(guild_id, uid) {
                                    should_recover = true;
                                    let _ = http
                                        .create_ban(Id::new(guild_id), Id::new(uid))
                                        .delete_message_seconds(0)
                                        .await;

                                    tokio::spawn(async move {
                                        let _ = sqlx::query!(
                                            "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                            guild_id as i64, uid as i64, "channel_delete", 100, "ban", 1
                                        ).execute(&db.pool).await;
                                    });
                                }
                            }

                            if should_recover
                                && let Some(snap) =
                                    engine().get_channel_snap(guild_id, channel_id).await
                                && engine()
                                    .mark_recovering(guild_id, "channel", channel_id)
                                    .await
                            {
                                recovery()
                                    .enqueue(snapshot::RecoveryTask {
                                        guild_id,
                                        priority: snap.position,
                                        kind: snapshot::RecoveryKind::Channel(snap),
                                    })
                                    .await;
                            }
                        });

                        engine().remove_channel_snap(guild_id, channel_id).await;
                    }
                }
                twilight_model::gateway::event::Event::RoleDelete(ev) => {
                    let guild_id = ev.guild_id.get();
                    let role_id = ev.role_id.get();

                    let http = self.http.clone();
                    let db = self.db.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                        let audit_result = tokio::time::timeout(
                            std::time::Duration::from_millis(150),
                            http.audit_log(Id::new(guild_id))
                                .action_type(
                                    twilight_model::guild::audit_log::AuditLogEventType::RoleDelete,
                                )
                                .limit(1),
                        )
                        .await;

                        let mut should_recover = false;
                        if let Ok(Ok(resp)) = audit_result
                            && let Ok(audit) = resp.model().await
                            && let Some(entry) = audit.entries.first()
                            && let Some(user_id) = entry.user_id
                        {
                            let uid = user_id.get();

                            if should_skip(guild_id, uid) {
                                return;
                            }

                            if !engine().is_whitelisted(guild_id, uid) {
                                should_recover = true;
                                let _ = http
                                    .create_ban(Id::new(guild_id), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;

                                tokio::spawn(async move {
                                    let _ = sqlx::query!(
                                        "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                        guild_id as i64, uid as i64, "role_delete", 100, "ban", 1
                                    ).execute(&db.pool).await;
                                });
                            }
                        }

                        if should_recover
                            && let Some(snap) = engine().get_role_snap(guild_id, role_id).await
                            && snap.name != "@everyone"
                            && engine().mark_recovering(guild_id, "role", role_id).await
                        {
                            recovery()
                                .enqueue(snapshot::RecoveryTask {
                                    guild_id,
                                    priority: snap.position as i32,
                                    kind: snapshot::RecoveryKind::Role(snap),
                                })
                                .await;
                        }
                    });

                    engine().remove_role_snap(guild_id, role_id).await;
                }

                twilight_model::gateway::event::Event::RoleCreate(ev) => {
                    let snap = snapshot::RoleSnap {
                        id: ev.role.id.get(),
                        name: ev.role.name.clone(),
                        color: ev.role.colors.primary_color,
                        permissions: ev.role.permissions.bits(),
                        position: ev.role.position,
                        hoist: ev.role.hoist,
                        mentionable: ev.role.mentionable,
                    };
                    engine().upsert_role_snap(ev.guild_id.get(), snap).await;
                }

                twilight_model::gateway::event::Event::RoleUpdate(ev) => {
                    let snap = snapshot::RoleSnap {
                        id: ev.role.id.get(),
                        name: ev.role.name.clone(),
                        color: ev.role.colors.primary_color,
                        permissions: ev.role.permissions.bits(),
                        position: ev.role.position,
                        hoist: ev.role.hoist,
                        mentionable: ev.role.mentionable,
                    };
                    engine().upsert_role_snap(ev.guild_id.get(), snap).await;
                }

                twilight_model::gateway::event::Event::MemberAdd(ev) => {
                    engine()
                        .set_member_roles(
                            ev.guild_id.get(),
                            ev.user.id.get(),
                            ev.roles.iter().map(|r| r.get()).collect(),
                        )
                        .await;
                }

                twilight_model::gateway::event::Event::MemberUpdate(ev) => {
                    engine()
                        .set_member_roles(
                            ev.guild_id.get(),
                            ev.user.id.get(),
                            ev.roles.iter().map(|r| r.get()).collect(),
                        )
                        .await;
                }

                twilight_model::gateway::event::Event::MessageCreate(ev) => {
                    if let Some(gid) = ev.guild_id {
                        if ev.author.bot {
                            return Ok(());
                        }
                        let matches = engine()
                            .scan_content(gid.get(), ev.author.id.get(), &ev.content, &[])
                            .await;
                        let mut conn = _ctx.redis.get().await.unwrap();
                        for m in matches {
                            if let Some(result) = engine()
                                .process_event(
                                    gid.get(),
                                    ev.author.id.get(),
                                    m.module,
                                    None,
                                    &mut conn,
                                )
                                .await
                                && result.triggered
                            {
                                let _ = punishment::execute(
                                    engine(),
                                    &self.http,
                                    &self.db,
                                    gid.get(),
                                    ev.author.id.get(),
                                    &result,
                                    &mut conn,
                                )
                                .await;
                            }
                        }
                    }
                }

                twilight_model::gateway::event::Event::WebhooksUpdate(ev) => {
                    let guild_id = ev.guild_id.get();
                    let channel_id = ev.channel_id.get();

                    let http = self.http.clone();
                    let db = self.db.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                        let audit_result = tokio::time::timeout(
                            std::time::Duration::from_millis(150),
                            http.audit_log(Id::new(guild_id))
                                .action_type(twilight_model::guild::audit_log::AuditLogEventType::WebhookCreate)
                                .limit(1)
                        ).await;

                        if let Ok(Ok(resp)) = audit_result
                            && let Ok(audit) = resp.model().await
                            && let Some(entry) = audit.entries.first()
                            && let Some(user_id) = entry.user_id
                        {
                            let uid = user_id.get();

                            if should_skip(guild_id, uid) {
                                return;
                            }

                            if !engine().is_whitelisted(guild_id, uid) {
                                let _ = http
                                    .create_ban(Id::new(guild_id), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;

                                let webhooks = http.channel_webhooks(Id::new(channel_id)).await;
                                if let Ok(wh_response) = webhooks
                                    && let Ok(webhooks_list) = wh_response.models().await
                                {
                                    for webhook in webhooks_list {
                                        let _ = http.delete_webhook(webhook.id).await;
                                    }
                                }

                                tokio::spawn(async move {
                                    let _ = sqlx::query!(
                                        "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                        guild_id as i64, uid as i64, "webhook_create", 100, "ban", 1
                                    ).execute(&db.pool).await;
                                });
                            }
                        }
                    });
                }

                twilight_model::gateway::event::Event::GuildEmojisUpdate(ev) => {
                    let guild_id = ev.guild_id.get();

                    let http = self.http.clone();
                    let db = self.db.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                        let audit_result = tokio::time::timeout(
                            std::time::Duration::from_millis(150),
                            http.audit_log(Id::new(guild_id))
                                .action_type(twilight_model::guild::audit_log::AuditLogEventType::EmojiCreate)
                                .limit(1)
                        ).await;

                        if let Ok(Ok(resp)) = audit_result
                            && let Ok(audit) = resp.model().await
                            && let Some(entry) = audit.entries.first()
                            && let Some(user_id) = entry.user_id
                        {
                            let uid = user_id.get();

                            if should_skip(guild_id, uid) {
                                return;
                            }

                            if !engine().is_whitelisted(guild_id, uid) {
                                let _ = http
                                    .create_ban(Id::new(guild_id), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;

                                tokio::spawn(async move {
                                    let _ = sqlx::query!(
                                        "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                        guild_id as i64, uid as i64, "emoji_spam", 100, "ban", 1
                                    ).execute(&db.pool).await;
                                });
                            }
                        }
                    });
                }

                twilight_model::gateway::event::Event::GuildUpdate(ev) => {
                    let guild_id = ev.id.get();

                    let http = self.http.clone();
                    let db = self.db.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                        let audit_result = tokio::time::timeout(
                            std::time::Duration::from_millis(150),
                            http.audit_log(Id::new(guild_id))
                                .action_type(twilight_model::guild::audit_log::AuditLogEventType::GuildUpdate)
                                .limit(1)
                        ).await;

                        if let Ok(Ok(resp)) = audit_result
                            && let Ok(audit) = resp.model().await
                            && let Some(entry) = audit.entries.first()
                            && let Some(user_id) = entry.user_id
                        {
                            let uid = user_id.get();

                            if should_skip(guild_id, uid) {
                                return;
                            }

                            if !engine().is_whitelisted(guild_id, uid) {
                                let _ = http
                                    .create_ban(Id::new(guild_id), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;

                                tokio::spawn(async move {
                                    let _ = sqlx::query!(
                                        "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                        guild_id as i64, uid as i64, "guild_update", 100, "ban", 1
                                    ).execute(&db.pool).await;
                                });
                            }
                        }
                    });
                }

                twilight_model::gateway::event::Event::IntegrationCreate(ev) => {
                    if let Some(gid) = ev.guild_id {
                        let guild_id = gid.get();

                        let http = self.http.clone();
                        let db = self.db.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                            let audit_result = tokio::time::timeout(
                                std::time::Duration::from_millis(150),
                                http.audit_log(Id::new(guild_id))
                                    .action_type(twilight_model::guild::audit_log::AuditLogEventType::IntegrationCreate)
                                    .limit(1)
                            ).await;

                            if let Ok(Ok(resp)) = audit_result
                                && let Ok(audit) = resp.model().await
                                && let Some(entry) = audit.entries.first()
                                && let Some(user_id) = entry.user_id
                            {
                                let uid = user_id.get();

                                if should_skip(guild_id, uid) {
                                    return;
                                }

                                if !engine().is_whitelisted(guild_id, uid) {
                                    let _ = http
                                        .create_ban(Id::new(guild_id), Id::new(uid))
                                        .delete_message_seconds(0)
                                        .await;

                                    tokio::spawn(async move {
                                        let _ = sqlx::query!(
                                            "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                            guild_id as i64, uid as i64, "integration_create", 100, "ban", 1
                                        ).execute(&db.pool).await;
                                    });
                                }
                            }
                        });
                    }
                }

                twilight_model::gateway::event::Event::MemberRemove(ev) => {
                    let guild_id = ev.guild_id.get();
                    let _user_id = ev.user.id.get();

                    let http = self.http.clone();
                    let db = self.db.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                        let audit_result = tokio::time::timeout(
                            std::time::Duration::from_millis(150),
                            http.audit_log(Id::new(guild_id))
                                .action_type(twilight_model::guild::audit_log::AuditLogEventType::MemberPrune)
                                .limit(1)
                        ).await;

                        if let Ok(Ok(resp)) = audit_result
                            && let Ok(audit) = resp.model().await
                            && let Some(entry) = audit.entries.first()
                            && let Some(executor_id) = entry.user_id
                        {
                            let uid = executor_id.get();

                            if should_skip(guild_id, uid) {
                                return;
                            }

                            if !engine().is_whitelisted(guild_id, uid) {
                                let _ = http
                                    .create_ban(Id::new(guild_id), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;

                                tokio::spawn(async move {
                                    let _ = sqlx::query!(
                                        "INSERT INTO antinuke_incident_log (guild_id, user_id, action_type, score, punishment, count_in_window) VALUES ($1, $2, $3, $4, $5, $6)",
                                        guild_id as i64, uid as i64, "member_prune", 100, "ban", 1
                                    ).execute(&db.pool).await;
                                });
                            }
                        }
                    });
                }

                twilight_model::gateway::event::Event::GuildAuditLogEntryCreate(ev) => {
                    if let Some(gid) = ev.guild_id
                        && let Some(user_id) = ev.user_id
                    {
                        let uid = user_id.get();
                        let gid_val = gid.get();

                        let should_ban = matches!(
                            ev.action_type,
                            twilight_model::guild::audit_log::AuditLogEventType::ChannelDelete
                                | twilight_model::guild::audit_log::AuditLogEventType::RoleDelete
                                | twilight_model::guild::audit_log::AuditLogEventType::MemberKick
                                | twilight_model::guild::audit_log::AuditLogEventType::MemberBanAdd
                                | twilight_model::guild::audit_log::AuditLogEventType::WebhookCreate
                                | twilight_model::guild::audit_log::AuditLogEventType::EmojiCreate
                                | twilight_model::guild::audit_log::AuditLogEventType::GuildUpdate
                                | twilight_model::guild::audit_log::AuditLogEventType::IntegrationCreate
                                | twilight_model::guild::audit_log::AuditLogEventType::MemberPrune
                        );

                        if should_ban
                            && !should_skip(gid_val, uid)
                            && !engine().is_whitelisted(gid_val, uid)
                        {
                            if engine().ban(gid_val, uid) {
                                return Ok(());
                            }
                            let http = self.http.clone();
                            tokio::spawn(async move {
                                let _ = http
                                    .create_ban(Id::new(gid_val), Id::new(uid))
                                    .delete_message_seconds(0)
                                    .await;
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
