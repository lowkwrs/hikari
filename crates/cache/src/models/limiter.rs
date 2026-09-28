use hikari_utils::error::HikariError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub struct Limiter {}

impl Limiter {
    pub async fn check_and_increment(
        conn: &mut MultiplexedConnection,
        key: &str,
        window_seconds: u64,
    ) -> Result<i64, HikariError> {
        let count: i64 = conn.incr(key, 1).await.map_err(HikariError::Cache)?;

        if count == 1 {
            let _: () = conn
                .expire(key, window_seconds as i64)
                .await
                .map_err(HikariError::Cache)?;
        }

        Ok(count)
    }
}
