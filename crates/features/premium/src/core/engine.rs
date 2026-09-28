use dashmap::DashMap;
use dashmap::DashSet;
use hikari_database::pool::Database;
use hikari_database::repository::premium_repository::PremiumRepository;
use std::sync::Arc;

pub struct PremiumEngine {
    repo: Arc<PremiumRepository>,
    cache: Arc<DashMap<i64, bool>>,
    noprefix_cache: Arc<DashMap<i64, bool>>,
    /// Per-guild set of user IDs with noprefix access: guild_id -> Set<user_id>
    noprefix_user_cache: Arc<DashMap<i64, DashSet<i64>>>,
}

impl PremiumEngine {
    pub fn new(db: Database) -> Self {
        Self {
            repo: Arc::new(PremiumRepository::new(db.pool.clone())),
            cache: Arc::new(DashMap::new()),
            noprefix_cache: Arc::new(DashMap::new()),
            noprefix_user_cache: Arc::new(DashMap::new()),
        }
    }

    pub fn is_owner(user_id: u64) -> bool {
        let owners = std::env::var("OWNER").unwrap_or_default();
        let owner_ids: Vec<u64> = owners
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();
        owner_ids.contains(&user_id)
    }

    #[inline(always)]
    pub async fn is_user_premium(&self, user_id: i64) -> bool {
        if Self::is_owner(user_id as u64) {
            return true;
        }

        if let Some(cached) = self.cache.get(&user_id) {
            return *cached;
        }

        let is_premium = self.repo.is_user_premium(user_id).await.unwrap_or(false);
        self.cache.insert(user_id, is_premium);
        is_premium
    }

    #[inline(always)]
    pub async fn is_guild_premium(&self, guild_id: i64) -> bool {
        if let Some(cached) = self.cache.get(&guild_id) {
            return *cached;
        }

        let is_premium = self.repo.is_guild_premium(guild_id).await.unwrap_or(false);
        self.cache.insert(guild_id, is_premium);
        is_premium
    }

    #[inline(always)]
    pub async fn is_noprefix_enabled(&self, guild_id: i64) -> bool {
        if let Some(cached) = self.noprefix_cache.get(&guild_id) {
            return *cached;
        }

        let enabled = self
            .repo
            .is_noprefix_enabled(guild_id)
            .await
            .unwrap_or(false);
        self.noprefix_cache.insert(guild_id, enabled);
        enabled
    }

    /// Check if a user has noprefix access in a guild.
    /// Bot owners and guild owners always have access when noprefix is enabled.
    pub async fn has_noprefix_access(
        &self,
        guild_id: i64,
        user_id: i64,
        guild_owner_id: Option<u64>,
    ) -> bool {
        if Self::is_owner(user_id as u64) {
            return true;
        }

        if let Some(owner_id) = guild_owner_id
            && user_id as u64 == owner_id
        {
            return true;
        }

        if let Some(user_set) = self.noprefix_user_cache.get(&guild_id) {
            return user_set.contains(&user_id);
        }

        let users = self
            .repo
            .get_noprefix_users(guild_id)
            .await
            .unwrap_or_default();
        let set = DashSet::new();
        for uid in &users {
            set.insert(*uid);
        }
        let has_access = set.contains(&user_id);
        self.noprefix_user_cache.insert(guild_id, set);
        has_access
    }

    pub async fn add_noprefix_user(
        &self,
        guild_id: i64,
        user_id: i64,
        added_by: i64,
    ) -> Result<(), sqlx::Error> {
        self.repo
            .add_noprefix_user(guild_id, user_id, added_by)
            .await?;
        self.noprefix_user_cache
            .entry(guild_id)
            .or_default()
            .insert(user_id);
        Ok(())
    }

    pub async fn remove_noprefix_user(
        &self,
        guild_id: i64,
        user_id: i64,
    ) -> Result<(), sqlx::Error> {
        self.repo.remove_noprefix_user(guild_id, user_id).await?;
        if let Some(set) = self.noprefix_user_cache.get(&guild_id) {
            set.remove(&user_id);
        }
        Ok(())
    }

    pub async fn get_noprefix_users(&self, guild_id: i64) -> Vec<i64> {
        self.repo
            .get_noprefix_users(guild_id)
            .await
            .unwrap_or_default()
    }

    pub async fn activate_guild(&self, guild_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        self.repo.activate_guild(guild_id, user_id).await?;
        self.cache.insert(guild_id, true);
        Ok(())
    }

    pub async fn revoke_guild(&self, guild_id: i64) -> Result<(), sqlx::Error> {
        self.repo.revoke_guild(guild_id).await?;
        self.repo.clear_noprefix_users(guild_id).await.ok();
        self.cache.remove(&guild_id);
        self.noprefix_cache.remove(&guild_id);
        self.noprefix_user_cache.remove(&guild_id);
        Ok(())
    }

    pub async fn revoke_user(&self, user_id: i64) -> Result<(), sqlx::Error> {
        let guilds = self.repo.get_user_guilds(user_id).await.unwrap_or_default();
        self.repo.remove_user(user_id).await?;

        self.cache.remove(&user_id);
        for guild_id in guilds {
            self.cache.remove(&guild_id);
            self.noprefix_cache.remove(&guild_id);
            self.noprefix_user_cache.remove(&guild_id);
        }

        Ok(())
    }

    pub async fn toggle_noprefix(&self, guild_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
        self.repo.toggle_noprefix(guild_id, enabled).await?;
        self.noprefix_cache.insert(guild_id, enabled);
        Ok(())
    }

    pub async fn add_user(&self, user_id: i64, activated_by: i64) -> Result<(), sqlx::Error> {
        self.repo.add_user(user_id, activated_by).await?;
        self.cache.insert(user_id, true);
        Ok(())
    }

    pub fn invalidate_cache(&self, id: i64) {
        self.cache.remove(&id);
        self.noprefix_cache.remove(&id);
        self.noprefix_user_cache.remove(&id);
    }
}
