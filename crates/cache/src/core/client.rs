use hikari_utils::error::HikariError;
pub use redis::Client as RedisClient;
use redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct Client {
    pub connection: MultiplexedConnection,
}

impl Client {
    pub async fn connect(url: &str) -> Result<Self, HikariError> {
        let client = RedisClient::open(url).map_err(HikariError::Cache)?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(HikariError::Cache)?;

        Ok(Self { connection })
    }
}
