use crate::core::pool::RedisPool;
use hikari_utils::error::HikariError;
use moka::future::Cache;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

#[derive(Clone)]
pub struct HybridCache<V> {
    l1: Cache<String, V>,
    l2: RedisPool,
    ttl_secs: u64,
}

impl<V> HybridCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(l2: RedisPool, l1_capacity: u64, ttl_secs: u64) -> Self {
        let l1 = Cache::builder()
            .max_capacity(l1_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .initial_capacity((l1_capacity / 10) as usize)
            .build();
        Self { l1, l2, ttl_secs }
    }

    #[inline]
    pub async fn get(&self, key: &str) -> Result<Option<V>, HikariError> {
        if let Some(val) = self.l1.get(key).await {
            return Ok(Some(val));
        }

        let mut conn = self.l2.get_connection().await?;
        let bytes: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(HikariError::Cache)?;

        if let Some(data) = bytes
            && let Ok(val) = bincode::deserialize::<V>(&data)
        {
            self.l1.insert(key.to_string(), val.clone()).await;
            return Ok(Some(val));
        }

        Ok(None)
    }

    #[inline]
    pub async fn set(&self, key: &str, value: V) -> Result<(), HikariError> {
        self.l1.insert(key.to_string(), value.clone()).await;

        if let Ok(bytes) = bincode::serialize(&value)
            && let Ok(mut conn) = self.l2.get_connection().await
        {
            let _: Result<(), _> = redis::cmd("SET")
                .arg(key)
                .arg(bytes)
                .arg("EX")
                .arg(self.ttl_secs)
                .query_async(&mut conn)
                .await;
        }
        Ok(())
    }

    #[inline]
    pub async fn invalidate(&self, key: &str) -> Result<(), HikariError> {
        self.l1.invalidate(key).await;
        if let Ok(mut conn) = self.l2.get_connection().await {
            let _: Result<(), _> = redis::cmd("DEL").arg(key).query_async(&mut conn).await;
        }
        Ok(())
    }
}
