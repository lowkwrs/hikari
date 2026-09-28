use deadpool_redis::{Config, CreatePoolError, Pool, Runtime};
use hikari_utils::error::HikariError;

#[derive(Clone)]
pub struct RedisPool {
    pub pool: Pool,
}

impl RedisPool {
    pub fn new(url: &str) -> Result<Self, HikariError> {
        let cfg = Config::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e: CreatePoolError| HikariError::Internal(e.to_string()))?;

        Ok(Self { pool })
    }

    #[inline]
    pub async fn get_connection(&self) -> Result<deadpool_redis::Connection, HikariError> {
        self.pool
            .get()
            .await
            .map_err(|e| HikariError::Internal(e.to_string()))
    }
}
