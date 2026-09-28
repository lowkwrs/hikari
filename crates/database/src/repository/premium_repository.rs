use crate::models::premium::PremiumGuild;
use sqlx::{PgPool, Row};

pub struct PremiumRepository {
    pool: PgPool,
}

impl PremiumRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_user(&self, user_id: i64, activated_by: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO premium_users (user_id, activated_by) VALUES ($1, $2) ON CONFLICT (user_id) DO NOTHING"
        )
        .bind(user_id)
        .bind(activated_by)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_user(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM premium_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM premium_guilds WHERE activated_by = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_user_premium(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let result: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM premium_users WHERE user_id = $1)")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(result.0)
    }

    pub async fn activate_guild(&self, guild_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO premium_guilds (guild_id, activated_by) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET activated_by = $2, activated_at = NOW()"
        )
        .bind(guild_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn revoke_guild(&self, guild_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM premium_guilds WHERE guild_id = $1")
            .bind(guild_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_guild_premium(&self, guild_id: i64) -> Result<bool, sqlx::Error> {
        let result: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM premium_guilds WHERE guild_id = $1)")
                .bind(guild_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(result.0)
    }

    pub async fn toggle_noprefix(&self, guild_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE premium_guilds SET noprefix_enabled = $1 WHERE guild_id = $2")
            .bind(enabled)
            .bind(guild_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_noprefix_enabled(&self, guild_id: i64) -> Result<bool, sqlx::Error> {
        let result: Option<(bool,)> =
            sqlx::query_as("SELECT noprefix_enabled FROM premium_guilds WHERE guild_id = $1")
                .bind(guild_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(result.map(|r| r.0).unwrap_or(false))
    }

    pub async fn get_user_guilds(&self, user_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let rows: Vec<(i64,)> =
            sqlx::query_as("SELECT guild_id FROM premium_guilds WHERE activated_by = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn get_guild_info(&self, guild_id: i64) -> Result<Option<PremiumGuild>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT guild_id, activated_by, activated_at, noprefix_enabled FROM premium_guilds WHERE guild_id = $1"
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| PremiumGuild {
            guild_id: r.try_get("guild_id").unwrap(),
            activated_by: r.try_get("activated_by").unwrap(),
            activated_at: r.try_get("activated_at").unwrap(),
            noprefix_enabled: r.try_get("noprefix_enabled").unwrap(),
        }))
    }

    pub async fn add_noprefix_user(
        &self,
        guild_id: i64,
        user_id: i64,
        added_by: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO noprefix_users (guild_id, user_id, added_by) VALUES ($1, $2, $3) ON CONFLICT (guild_id, user_id) DO NOTHING"
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(added_by)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_noprefix_user(
        &self,
        guild_id: i64,
        user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM noprefix_users WHERE guild_id = $1 AND user_id = $2")
            .bind(guild_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_noprefix_user(&self, guild_id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM noprefix_users WHERE guild_id = $1 AND user_id = $2)",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    pub async fn get_noprefix_users(&self, guild_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let rows: Vec<(i64,)> =
            sqlx::query_as("SELECT user_id FROM noprefix_users WHERE guild_id = $1")
                .bind(guild_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn clear_noprefix_users(&self, guild_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM noprefix_users WHERE guild_id = $1")
            .bind(guild_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
