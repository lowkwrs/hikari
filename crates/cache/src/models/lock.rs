use hikari_utils::error::HikariError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub struct Lock {}

impl Lock {
    pub async fn acquire(
        conn: &mut MultiplexedConnection,
        key: &str,
        ttl_millis: u64,
    ) -> Result<bool, HikariError> {
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg("1")
            .arg("NX")
            .arg("PX")
            .arg(ttl_millis)
            .query_async(conn)
            .await
            .map_err(HikariError::Cache)?;

        Ok(result.is_some())
    }

    pub async fn release(conn: &mut MultiplexedConnection, key: &str) -> Result<(), HikariError> {
        let _: () = conn.del(key).await.map_err(HikariError::Cache)?;
        Ok(())
    }
}
