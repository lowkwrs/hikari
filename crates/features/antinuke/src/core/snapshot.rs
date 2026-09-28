use dashmap::DashMap;
use hikari_cache::RedisPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermOverwrite {
    pub id: u64,
    pub kind: u8,
    pub allow: u64,
    pub deny: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryTask {
    pub guild_id: u64,
    pub priority: i32,
    pub kind: RecoveryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryKind {
    Channel(ChannelSnap),
    Role(RoleSnap),
    MemberRoles { user_id: u64, role_ids: Vec<u64> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelSnap {
    pub id: u64,
    pub name: String,
    pub kind: u8,
    pub position: i32,
    pub topic: Option<String>,
    pub nsfw: bool,
    pub rate_limit_per_user: u16,
    pub parent_id: Option<u64>,
    pub overwrites: Vec<PermOverwrite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleSnap {
    pub id: u64,
    pub name: String,
    pub color: u32,
    pub permissions: u64,
    pub position: i64,
    pub hoist: bool,
    pub mentionable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildSnapshot {
    pub guild_id: u64,
    pub name: String,
    pub channels: Vec<ChannelSnap>,
    pub roles: Vec<RoleSnap>,
}

#[derive(Clone)]
pub struct SnapshotStore {
    channels: Arc<DashMap<(u64, u64), ChannelSnap>>,
    roles: Arc<DashMap<(u64, u64), RoleSnap>>,
    members: Arc<DashMap<(u64, u64), Vec<u64>>>,
    role_members: Arc<DashMap<(u64, u64), Vec<u64>>>,
    in_recovery: Arc<DashMap<(u64, String, u64), ()>>,
}

impl SnapshotStore {
    pub fn new(_redis_pool: RedisPool) -> Self {
        Self {
            channels: Arc::new(DashMap::with_capacity(5000)),
            roles: Arc::new(DashMap::with_capacity(2000)),
            members: Arc::new(DashMap::with_capacity(50000)),
            role_members: Arc::new(DashMap::with_capacity(10000)),
            in_recovery: Arc::new(DashMap::with_capacity(1000)),
        }
    }

    #[inline]
    pub fn mark_recovering(&self, guild_id: u64, kind: &str, resource_id: u64) -> bool {
        self.in_recovery
            .insert((guild_id, kind.to_string(), resource_id), ())
            .is_none()
    }

    #[inline]
    pub fn clear_recovering(&self, guild_id: u64, kind: &str, resource_id: u64) {
        self.in_recovery
            .remove(&(guild_id, kind.to_string(), resource_id));
    }

    pub async fn set(&self, snap: GuildSnapshot) {
        let gid = snap.guild_id;
        for ch in snap.channels {
            self.channels.insert((gid, ch.id), ch);
        }
        for r in snap.roles {
            self.roles.insert((gid, r.id), r);
        }
    }

    #[inline]
    pub async fn get_channel(&self, guild_id: u64, channel_id: u64) -> Option<ChannelSnap> {
        self.channels
            .get(&(guild_id, channel_id))
            .map(|v| v.clone())
    }

    #[inline]
    pub async fn upsert_channel(&self, guild_id: u64, channel: ChannelSnap) {
        self.channels.insert((guild_id, channel.id), channel);
    }

    #[inline]
    pub async fn remove_channel(&self, guild_id: u64, channel_id: u64) {
        self.channels.remove(&(guild_id, channel_id));
    }

    #[inline]
    pub async fn get_role(&self, guild_id: u64, role_id: u64) -> Option<RoleSnap> {
        self.roles.get(&(guild_id, role_id)).map(|v| v.clone())
    }

    #[inline]
    pub async fn get_role_perms(&self, guild_id: u64, role_id: u64) -> u64 {
        self.roles
            .get(&(guild_id, role_id))
            .map(|r| r.permissions)
            .unwrap_or(0)
    }

    #[inline]
    pub async fn upsert_role(&self, guild_id: u64, role: RoleSnap) {
        self.roles.insert((guild_id, role.id), role);
    }

    #[inline]
    pub async fn remove_role(&self, guild_id: u64, role_id: u64) {
        self.roles.remove(&(guild_id, role_id));
    }

    #[inline]
    pub async fn set_member_roles(&self, guild_id: u64, user_id: u64, roles: Vec<u64>) {
        for &role_id in &roles {
            self.role_members
                .entry((guild_id, role_id))
                .or_default()
                .push(user_id);
        }
        self.members.insert((guild_id, user_id), roles);
    }

    #[inline]
    pub async fn get_member_roles(&self, guild_id: u64, user_id: u64) -> Option<Vec<u64>> {
        self.members.get(&(guild_id, user_id)).map(|v| v.clone())
    }

    #[inline]
    pub async fn get_role_members(&self, guild_id: u64, role_id: u64) -> Vec<u64> {
        self.role_members
            .get(&(guild_id, role_id))
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub async fn remove_guild(&self, guild_id: u64) {
        self.channels.retain(|(gid, _), _| *gid != guild_id);
        self.roles.retain(|(gid, _), _| *gid != guild_id);
        self.members.retain(|(gid, _), _| *gid != guild_id);
        self.role_members.retain(|(gid, _), _| *gid != guild_id);
    }
}
