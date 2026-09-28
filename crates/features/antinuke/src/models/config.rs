use hikari_utils::ids::UserId;

pub struct AntinukeConfig {
    pub ban_threshold: u32,
    pub channel_delete_threshold: u32,
    pub whitelisted_users: Vec<UserId>,
}

impl Default for AntinukeConfig {
    fn default() -> Self {
        Self {
            ban_threshold: 5,
            channel_delete_threshold: 3,
            whitelisted_users: Vec::new(),
        }
    }
}
