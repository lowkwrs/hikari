use crate::models::antinuke_config::{AntinukeConfig, AntinukeModuleConfigRow};
use hikari_utils::error::HikariError;
use sqlx::PgPool;

pub struct AntinukeRepository {
    pool: PgPool,
}

impl AntinukeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_config(&self, guild_id: i64) -> Result<Option<AntinukeConfig>, HikariError> {
        let config = sqlx::query_as!(
            AntinukeConfig,
            r#"
            SELECT guild_id, enabled, log_channel_id, updated_at
            FROM antinuke_guild_config
            WHERE guild_id = $1
            "#,
            guild_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(config)
    }

    pub async fn get_module_configs(
        &self,
        guild_id: i64,
    ) -> Result<Vec<AntinukeModuleConfigRow>, HikariError> {
        let rows = sqlx::query_as!(
            AntinukeModuleConfigRow,
            r#"
            SELECT guild_id, action_type, enabled, threshold, window_secs, punishment, log_only
            FROM antinuke_module_config
            WHERE guild_id = $1
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(rows)
    }

    pub async fn upsert_config(&self, config: &AntinukeConfig) -> Result<(), HikariError> {
        sqlx::query!(
            r#"
            INSERT INTO antinuke_guild_config (guild_id, enabled, log_channel_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (guild_id) DO UPDATE SET
                enabled = EXCLUDED.enabled,
                log_channel_id = EXCLUDED.log_channel_id,
                updated_at = now()
            "#,
            config.guild_id,
            config.enabled,
            config.log_channel_id
        )
        .execute(&self.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(())
    }

    pub async fn upsert_module_config(
        &self,
        config: &AntinukeModuleConfigRow,
    ) -> Result<(), HikariError> {
        sqlx::query!(
            r#"
            INSERT INTO antinuke_module_config (guild_id, action_type, enabled, threshold, window_secs, punishment, log_only)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (guild_id, action_type) DO UPDATE SET
                enabled = EXCLUDED.enabled,
                threshold = EXCLUDED.threshold,
                window_secs = EXCLUDED.window_secs,
                punishment = EXCLUDED.punishment,
                log_only = EXCLUDED.log_only
            "#,
            config.guild_id,
            config.action_type,
            config.enabled,
            config.threshold,
            config.window_secs,
            config.punishment,
            config.log_only
        )
        .execute(&self.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(())
    }

    pub async fn is_whitelisted(&self, guild_id: i64, user_id: i64) -> Result<bool, HikariError> {
        let result = sqlx::query!(
            r#"
            SELECT 1 as "exists!"
            FROM antinuke_whitelist
            WHERE guild_id = $1 AND user_id = $2
            "#,
            guild_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(result.is_some())
    }

    pub async fn add_whitelist(
        &self,
        guild_id: i64,
        user_id: i64,
        added_by: i64,
    ) -> Result<(), HikariError> {
        sqlx::query!(
            r#"
            INSERT INTO antinuke_whitelist (guild_id, user_id, added_by)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            guild_id,
            user_id,
            added_by
        )
        .execute(&self.pool)
        .await
        .map_err(HikariError::Database)?;
        Ok(())
    }

    pub async fn remove_whitelist(&self, guild_id: i64, user_id: i64) -> Result<(), HikariError> {
        sqlx::query!(
            r#"
            DELETE FROM antinuke_whitelist
            WHERE guild_id = $1 AND user_id = $2
            "#,
            guild_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(HikariError::Database)?;
        Ok(())
    }

    pub async fn get_whitelist_count(&self, guild_id: i64) -> Result<i64, HikariError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT count(*)
            FROM antinuke_whitelist
            WHERE guild_id = $1
            "#,
            guild_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(HikariError::Database)?;
        Ok(count.unwrap_or(0))
    }

    pub async fn get_whitelist(&self, guild_id: i64) -> Result<Vec<i64>, HikariError> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT user_id
            FROM antinuke_whitelist
            WHERE guild_id = $1
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(HikariError::Database)?;
        Ok(rows)
    }
}
