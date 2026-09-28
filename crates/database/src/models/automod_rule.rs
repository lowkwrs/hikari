use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct AutomodRule {
    pub id: i32,
    pub guild_id: i64,
    pub name: String,
    pub trigger_type: i16,
    pub action_type: i16,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    Spam = 1,
    AntiLink = 2,
    GhostPing = 3,
}

impl TryFrom<i16> for TriggerType {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TriggerType::Spam),
            2 => Ok(TriggerType::AntiLink),
            3 => Ok(TriggerType::GhostPing),
            _ => Err("Invalid TriggerType"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    DeleteMessage = 1,
    Timeout = 2,
    DeleteAndTimeout = 3,
}

impl TryFrom<i16> for ActionType {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ActionType::DeleteMessage),
            2 => Ok(ActionType::Timeout),
            3 => Ok(ActionType::DeleteAndTimeout),
            _ => Err("Invalid ActionType"),
        }
    }
}
