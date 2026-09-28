use dashmap::DashMap;
use hikari_cache::RedisPool;
use std::sync::Arc;

pub struct WhitelistStore {
    cache: Arc<DashMap<u64, Arc<DashMap<u64, ()>>>>,
}

impl Clone for WhitelistStore {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl WhitelistStore {
    pub fn new(_redis_pool: RedisPool) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn set(&self, guild_id: u64, user_ids: Vec<u64>) {
        let guild_cache = Arc::new(DashMap::new());
        for uid in user_ids {
            guild_cache.insert(uid, ());
        }
        self.cache.insert(guild_id, guild_cache);
    }

    #[inline(always)]
    pub fn is_whitelisted(&self, guild_id: u64, user_id: u64) -> bool {
        self.cache
            .get(&guild_id)
            .map(|g| g.contains_key(&user_id))
            .unwrap_or(false)
    }

    pub async fn add(&self, guild_id: u64, user_id: u64) {
        if let Some(guild_cache) = self.cache.get(&guild_id) {
            guild_cache.insert(user_id, ());
        } else {
            let guild_cache = Arc::new(DashMap::new());
            guild_cache.insert(user_id, ());
            self.cache.insert(guild_id, guild_cache);
        }
    }

    pub async fn remove(&self, guild_id: u64, user_id: u64) {
        if let Some(guild_cache) = self.cache.get(&guild_id) {
            guild_cache.remove(&user_id);
        }
    }

    pub async fn remove_guild(&self, guild_id: u64) {
        self.cache.remove(&guild_id);
    }
}
