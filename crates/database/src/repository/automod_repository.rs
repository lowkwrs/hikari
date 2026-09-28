use crate::models::automod_rule::AutomodRule;
use sqlx::PgPool;

pub struct AutomodRepository {}

impl AutomodRepository {
    pub async fn get_rules(pool: &PgPool, guild_id: i64) -> Result<Vec<AutomodRule>, sqlx::Error> {
        let rules = sqlx::query_as!(
            AutomodRule,
            r#"
            SELECT id, guild_id, name, trigger_type, action_type, enabled, created_at, updated_at
            FROM automod_rules
            WHERE guild_id = $1 AND enabled = true
            "#,
            guild_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rules)
    }

    pub async fn get_rule(
        pool: &PgPool,
        guild_id: i64,
        trigger_type: i16,
    ) -> Result<Option<AutomodRule>, sqlx::Error> {
        let rule = sqlx::query_as!(
            AutomodRule,
            r#"
            SELECT id, guild_id, name, trigger_type, action_type, enabled, created_at, updated_at
            FROM automod_rules
            WHERE guild_id = $1 AND trigger_type = $2
            "#,
            guild_id,
            trigger_type
        )
        .fetch_optional(pool)
        .await?;

        Ok(rule)
    }

    pub async fn create_rule(
        pool: &PgPool,
        guild_id: i64,
        name: &str,
        trigger_type: i16,
        action_type: i16,
        enabled: bool,
    ) -> Result<AutomodRule, sqlx::Error> {
        let rule = sqlx::query_as!(
            AutomodRule,
            r#"
            INSERT INTO automod_rules (guild_id, name, trigger_type, action_type, enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, guild_id, name, trigger_type, action_type, enabled, created_at, updated_at
            "#,
            guild_id,
            name,
            trigger_type,
            action_type,
            enabled
        )
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn update_rule(pool: &PgPool, rule: &AutomodRule) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE automod_rules
            SET action_type = $1, enabled = $2, updated_at = NOW()
            WHERE id = $3
            "#,
            rule.action_type,
            rule.enabled,
            rule.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
