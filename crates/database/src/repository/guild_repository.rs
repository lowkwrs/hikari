use crate::models::guild::GuildConfig;
use crate::pool::Database;
use hikari_utils::error::HikariError;
use hikari_utils::ids::GuildId;

pub struct GuildRepository {
    db: Database,
}

impl GuildRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_config(&self, guild_id: GuildId) -> Result<Option<GuildConfig>, HikariError> {
        let result = sqlx::query_as::<_, GuildConfig>(
            "SELECT id, created_at, updated_at, premium_tier FROM guilds WHERE id = $1",
        )
        .bind(guild_id.get() as i64)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(result)
    }

    pub async fn upsert_config(
        &self,
        guild_id: GuildId,
        premium_tier: i16,
    ) -> Result<GuildConfig, HikariError> {
        let result = sqlx::query_as::<_, GuildConfig>(
            r#"
            INSERT INTO guilds (id, premium_tier)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET premium_tier = EXCLUDED.premium_tier, updated_at = NOW()
            RETURNING id, created_at, updated_at, premium_tier
            "#,
        )
        .bind(guild_id.get() as i64)
        .bind(premium_tier)
        .fetch_one(&self.db.pool)
        .await
        .map_err(HikariError::Database)?;

        Ok(result)
    }
}
