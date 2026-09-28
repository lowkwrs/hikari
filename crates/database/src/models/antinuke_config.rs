use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct AntinukeConfig {
    pub guild_id: i64,
    pub enabled: bool,
    pub log_channel_id: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AntinukeModuleConfigRow {
    pub guild_id: i64,
    pub action_type: String,
    pub enabled: bool,
    pub threshold: i32,
    pub window_secs: i32,
    pub punishment: String,
    pub log_only: bool,
}
