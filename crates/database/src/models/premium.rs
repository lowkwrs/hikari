use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PremiumUser {
    pub user_id: i64,
    pub activated_at: DateTime<Utc>,
    pub activated_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PremiumGuild {
    pub guild_id: i64,
    pub activated_by: i64,
    pub activated_at: DateTime<Utc>,
    pub noprefix_enabled: bool,
}
