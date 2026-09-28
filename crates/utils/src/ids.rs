use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};

pub type GuildId = Id<GuildMarker>;
pub type UserId = Id<UserMarker>;
pub type ChannelId = Id<ChannelMarker>;
pub type RoleId = Id<RoleMarker>;

use std::sync::atomic::{AtomicU64, Ordering};

pub static BOT_ID: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn get_bot_id() -> u64 {
    BOT_ID.load(Ordering::Relaxed)
}

#[inline]
pub fn set_bot_id(id: u64) {
    BOT_ID.store(id, Ordering::Relaxed);
}
