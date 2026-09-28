use hikari_cache::HybridCache;
use hikari_cache::RedisPool;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{
    snapshot::{ChannelSnap, GuildSnapshot, RoleSnap, SnapshotStore},
    whitelist::WhitelistStore,
};
use crate::models::types::{ActionType, ContentMatch, GuildConfig, Punishment, ThreatResult};
use crate::modules::{
    content::{ContentScanInput, scan},
    scorer::{compute_score, has_dangerous_perm_grant, resolve_punishment},
};
use redis::AsyncCommands;

pub struct AntiNukeEngine {
    guilds: HybridCache<GuildConfig>,
    snapshots: SnapshotStore,
    whitelist_store: WhitelistStore,
    redis_pool: RedisPool,
    ban_executor: Option<hikari_utils::executor::BanExecutor>,
}

impl AntiNukeEngine {
    pub fn new(redis_pool: RedisPool) -> Self {
        Self {
            guilds: HybridCache::new(redis_pool.clone(), 10000, 3600),
            snapshots: SnapshotStore::new(redis_pool.clone()),
            whitelist_store: WhitelistStore::new(redis_pool.clone()),
            redis_pool,
            ban_executor: None,
        }
    }

    pub fn with_ban_executor(mut self, http: std::sync::Arc<twilight_http::Client>) -> Self {
        self.ban_executor = Some(hikari_utils::executor::BanExecutor::new(http));
        self
    }

    #[inline(always)]
    pub fn ban(&self, guild_id: u64, user_id: u64) -> bool {
        if let Some(executor) = &self.ban_executor {
            executor.execute(guild_id, user_id);
            true
        } else {
            false
        }
    }

    pub async fn configure(&self, guild_id: u64, config: GuildConfig, whitelist_ids: Vec<u64>) {
        let key = format!("hikari:guild_config:{}", guild_id);
        let _ = self.guilds.set(&key, config).await;
        self.whitelist_store.set(guild_id, whitelist_ids).await;
    }

    pub async fn remove_guild(&self, guild_id: u64) {
        let key = format!("hikari:guild_config:{}", guild_id);
        let _ = self.guilds.invalidate(&key).await;
        self.snapshots.remove_guild(guild_id).await;
        self.whitelist_store.remove_guild(guild_id).await;
    }

    pub async fn clear_user(
        &self,
        guild_id: u64,
        user_id: u64,
        redis: &mut deadpool_redis::Connection,
    ) {
        let punish_key = format!("hikari:punishment:{}:{}", guild_id, user_id);
        let _: () = redis.del(punish_key).await.unwrap_or(());

        let mut pipe = redis::pipe();
        for action in 0..10 {
            let mut key = String::with_capacity(64);
            let _ = write!(
                key,
                "hikari:antinuke:windows:{}:{}:{}",
                guild_id, user_id, action
            );
            pipe.del(key);
        }
        let _: () = pipe.query_async(redis).await.unwrap_or(());
    }

    pub async fn try_claim_punishment(&self, guild_id: u64, user_id: u64) -> bool {
        let key = format!("hikari:punishment:{}:{}", guild_id, user_id);
        if let Ok(mut conn) = self.redis_pool.get_connection().await {
            // SETNX with 10 seconds expiry to prevent duplicate punishment claims
            let result: redis::RedisResult<bool> = redis::cmd("SET")
                .arg(&key)
                .arg(1)
                .arg("NX")
                .arg("EX")
                .arg(10)
                .query_async(&mut conn)
                .await;
            match result {
                Ok(true) => return true, // successfully claimed
                _ => return false,
            }
        }
        false
    }

    pub async fn process_event(
        &self,
        guild_id: u64,
        user_id: u64,
        action: ActionType,
        extra_data: Option<&(u64, u64)>,
        redis_conn: &mut deadpool_redis::Connection,
    ) -> Option<ThreatResult> {
        let key = format!("hikari:guild_config:{}", guild_id);
        let config = self.guilds.get(&key).await.unwrap_or(None)?;

        if !config.enabled {
            return None;
        }

        if self.whitelist_store.is_whitelisted(guild_id, user_id) {
            return None;
        }

        let module_cfg = match config.modules.get(&(action as u8)) {
            Some(m) if m.enabled => m,
            _ => return None,
        };

        if action == ActionType::RoleUpdate {
            if let Some(&(old_perms, new_perms)) = extra_data {
                if !has_dangerous_perm_grant(old_perms, new_perms) {
                    return None;
                }
            } else {
                return None;
            }
        }

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if action.is_instant() || module_cfg.window_secs == 0 {
            let score = compute_score(action, 1);
            let (punishment_out, reason) =
                resolve_punishment(module_cfg.punishment, action, 1, score);
            return Some(ThreatResult {
                score,
                triggered: !module_cfg.log_only,
                punishment: punishment_out,
                reason,
                action,
                should_restore: action.requires_restore(),
                count_in_window: 1,
            });
        }

        let mut key = String::with_capacity(64);
        let _ = write!(
            key,
            "hikari:antinuke:windows:{}:{}:{}",
            guild_id, user_id, action as u8
        );
        let window_ms = (module_cfg.window_secs as u64) * 1000;
        let cutoff_ms = now_ms.saturating_sub(window_ms);

        let mut pipe = redis::pipe();
        pipe.cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg(0)
            .arg(cutoff_ms)
            .ignore();
        pipe.cmd("ZADD").arg(&key).arg(now_ms).arg(now_ms).ignore();
        pipe.cmd("ZCARD").arg(&key);
        pipe.cmd("EXPIRE")
            .arg(&key)
            .arg(module_cfg.window_secs)
            .ignore();

        let result: redis::RedisResult<(usize,)> = pipe.query_async(redis_conn).await;

        let count = match result {
            Ok((c,)) => c,
            Err(_) => 1,
        };

        let mut score = compute_score(action, count);
        if module_cfg.threshold == 0 {
            score = 100;
        }
        let triggered = count >= module_cfg.threshold as usize;

        if !triggered {
            if action.requires_restore() {
                return Some(ThreatResult {
                    score,
                    triggered: false,
                    punishment: Punishment::None,
                    reason: String::new(),
                    action,
                    should_restore: true,
                    count_in_window: count as u32,
                });
            }
            return None;
        }

        let (punishment_out, reason) =
            resolve_punishment(module_cfg.punishment, action, count as u32, score);

        Some(ThreatResult {
            score,
            triggered: !module_cfg.log_only,
            punishment: punishment_out,
            reason,
            action,
            should_restore: action.requires_restore(),
            count_in_window: count as u32,
        })
    }

    pub async fn scan_content(
        &self,
        guild_id: u64,
        user_id: u64,
        content: &str,
        author_roles: &[u64],
    ) -> Vec<ContentMatch> {
        let key = format!("hikari:guild_config:{}", guild_id);
        let config = match self.guilds.get(&key).await.unwrap_or(None) {
            Some(c) => c,
            None => return Vec::new(),
        };

        if !config.enabled {
            return Vec::new();
        }

        if self.whitelist_store.is_whitelisted(guild_id, user_id) {
            return Vec::new();
        }

        let input = ContentScanInput {
            content,
            author_roles,
        };
        scan(&input, &config.modules)
    }

    pub async fn process_content_match(
        &self,
        guild_id: u64,
        user_id: u64,
        module: ActionType,
        redis_conn: &mut deadpool_redis::Connection,
    ) -> Option<ThreatResult> {
        self.process_event(guild_id, user_id, module, None, redis_conn)
            .await
    }

    pub async fn set_snapshot(&self, snap: GuildSnapshot) {
        self.snapshots.set(snap).await;
    }

    pub async fn get_channel_snap(&self, guild_id: u64, channel_id: u64) -> Option<ChannelSnap> {
        self.snapshots.get_channel(guild_id, channel_id).await
    }

    pub async fn get_role_snap(&self, guild_id: u64, role_id: u64) -> Option<RoleSnap> {
        self.snapshots.get_role(guild_id, role_id).await
    }

    pub async fn get_role_perms(&self, guild_id: u64, role_id: u64) -> u64 {
        self.snapshots.get_role_perms(guild_id, role_id).await
    }

    pub async fn upsert_channel_snap(&self, guild_id: u64, ch: ChannelSnap) {
        self.snapshots.upsert_channel(guild_id, ch).await;
    }

    pub async fn remove_channel_snap(&self, guild_id: u64, channel_id: u64) {
        self.snapshots.remove_channel(guild_id, channel_id).await;
    }

    pub async fn upsert_role_snap(&self, guild_id: u64, role: RoleSnap) {
        self.snapshots.upsert_role(guild_id, role).await;
    }

    pub async fn remove_role_snap(&self, guild_id: u64, role_id: u64) {
        self.snapshots.remove_role(guild_id, role_id).await;
    }

    pub async fn set_member_roles(&self, guild_id: u64, user_id: u64, roles: Vec<u64>) {
        self.snapshots
            .set_member_roles(guild_id, user_id, roles)
            .await;
    }

    pub async fn get_member_roles(&self, guild_id: u64, user_id: u64) -> Option<Vec<u64>> {
        self.snapshots.get_member_roles(guild_id, user_id).await
    }

    pub async fn get_role_members(&self, guild_id: u64, role_id: u64) -> Vec<u64> {
        self.snapshots.get_role_members(guild_id, role_id).await
    }

    pub async fn mark_recovering(&self, guild_id: u64, kind: &str, resource_id: u64) -> bool {
        self.snapshots.mark_recovering(guild_id, kind, resource_id)
    }

    pub async fn clear_recovering(&self, guild_id: u64, kind: &str, resource_id: u64) {
        self.snapshots.clear_recovering(guild_id, kind, resource_id)
    }

    pub async fn get_log_channel(&self, guild_id: u64) -> Option<u64> {
        let key = format!("hikari:guild_config:{}", guild_id);
        self.guilds
            .get(&key)
            .await
            .unwrap_or(None)
            .and_then(|g| g.log_channel_id)
    }

    pub async fn whitelist_add(&self, guild_id: u64, user_id: u64) {
        self.whitelist_store.add(guild_id, user_id).await;
    }

    pub async fn whitelist_remove(&self, guild_id: u64, user_id: u64) {
        self.whitelist_store.remove(guild_id, user_id).await;
    }

    #[inline(always)]
    pub fn is_whitelisted(&self, guild_id: u64, user_id: u64) -> bool {
        self.whitelist_store.is_whitelisted(guild_id, user_id)
    }
}
