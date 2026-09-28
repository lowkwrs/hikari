use std::collections::HashMap;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    BanAdd = 0,
    BanRemove = 1,
    MemberKick = 2,
    BotAdd = 3,
    MemberUpdate = 4,
    MemberPrune = 5,
    ChannelCreate = 6,
    ChannelDelete = 7,
    ChannelUpdate = 8,
    EmojiCreate = 9,
    EmojiDelete = 10,
    EmojiUpdate = 11,
    StickerCreate = 12,
    StickerDelete = 13,
    StickerUpdate = 14,
    EveryonePing = 15,
    HerePing = 16,
    LinkInMessage = 17,
    RolePing = 18,
    RoleCreate = 19,
    RoleDelete = 20,
    RoleUpdate = 21,
    IntegrationCreate = 22,
    IntegrationUpdate = 23,
    IntegrationDelete = 24,
    GuildUpdate = 25,
    AutomodRuleCreate = 26,
    AutomodRuleUpdate = 27,
    AutomodRuleDelete = 28,
    GuildEventCreate = 29,
    GuildEventUpdate = 30,
    GuildEventDelete = 31,
    WebhookCreate = 32,
    WebhookUpdate = 33,
}

impl ActionType {
    pub fn is_content(self) -> bool {
        matches!(
            self,
            Self::EveryonePing | Self::HerePing | Self::LinkInMessage | Self::RolePing
        )
    }

    pub fn is_instant(self) -> bool {
        matches!(self, Self::BotAdd | Self::GuildUpdate | Self::MemberPrune)
    }

    pub fn requires_restore(self) -> bool {
        matches!(
            self,
            Self::ChannelDelete | Self::RoleDelete | Self::GuildUpdate | Self::MemberPrune
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BanAdd => "BAN_ADD",
            Self::BanRemove => "BAN_REMOVE",
            Self::MemberKick => "MEMBER_KICK",
            Self::BotAdd => "BOT_ADD",
            Self::MemberUpdate => "MEMBER_UPDATE",
            Self::MemberPrune => "MEMBER_PRUNE",
            Self::ChannelCreate => "CHANNEL_CREATE",
            Self::ChannelDelete => "CHANNEL_DELETE",
            Self::ChannelUpdate => "CHANNEL_UPDATE",
            Self::EmojiCreate => "EMOJI_CREATE",
            Self::EmojiDelete => "EMOJI_DELETE",
            Self::EmojiUpdate => "EMOJI_UPDATE",
            Self::StickerCreate => "STICKER_CREATE",
            Self::StickerDelete => "STICKER_DELETE",
            Self::StickerUpdate => "STICKER_UPDATE",
            Self::EveryonePing => "EVERYONE_PING",
            Self::HerePing => "HERE_PING",
            Self::LinkInMessage => "LINK_IN_MESSAGE",
            Self::RolePing => "ROLE_PING",
            Self::RoleCreate => "ROLE_CREATE",
            Self::RoleDelete => "ROLE_DELETE",
            Self::RoleUpdate => "ROLE_UPDATE",
            Self::IntegrationCreate => "INTEGRATION_CREATE",
            Self::IntegrationUpdate => "INTEGRATION_UPDATE",
            Self::IntegrationDelete => "INTEGRATION_DELETE",
            Self::GuildUpdate => "GUILD_UPDATE",
            Self::AutomodRuleCreate => "AUTOMOD_RULE_CREATE",
            Self::AutomodRuleUpdate => "AUTOMOD_RULE_UPDATE",
            Self::AutomodRuleDelete => "AUTOMOD_RULE_DELETE",
            Self::GuildEventCreate => "GUILD_EVENT_CREATE",
            Self::GuildEventUpdate => "GUILD_EVENT_UPDATE",
            Self::GuildEventDelete => "GUILD_EVENT_DELETE",
            Self::WebhookCreate => "WEBHOOK_CREATE",
            Self::WebhookUpdate => "WEBHOOK_UPDATE",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "BAN_ADD" => Some(Self::BanAdd),
            "BAN_REMOVE" => Some(Self::BanRemove),
            "MEMBER_KICK" => Some(Self::MemberKick),
            "BOT_ADD" => Some(Self::BotAdd),
            "MEMBER_UPDATE" => Some(Self::MemberUpdate),
            "MEMBER_PRUNE" => Some(Self::MemberPrune),
            "CHANNEL_CREATE" => Some(Self::ChannelCreate),
            "CHANNEL_DELETE" => Some(Self::ChannelDelete),
            "CHANNEL_UPDATE" => Some(Self::ChannelUpdate),
            "EMOJI_CREATE" => Some(Self::EmojiCreate),
            "EMOJI_DELETE" => Some(Self::EmojiDelete),
            "EMOJI_UPDATE" => Some(Self::EmojiUpdate),
            "STICKER_CREATE" => Some(Self::StickerCreate),
            "STICKER_DELETE" => Some(Self::StickerDelete),
            "STICKER_UPDATE" => Some(Self::StickerUpdate),
            "EVERYONE_PING" => Some(Self::EveryonePing),
            "HERE_PING" => Some(Self::HerePing),
            "LINK_IN_MESSAGE" => Some(Self::LinkInMessage),
            "ROLE_PING" => Some(Self::RolePing),
            "ROLE_CREATE" => Some(Self::RoleCreate),
            "ROLE_DELETE" => Some(Self::RoleDelete),
            "ROLE_UPDATE" => Some(Self::RoleUpdate),
            "INTEGRATION_CREATE" => Some(Self::IntegrationCreate),
            "INTEGRATION_UPDATE" => Some(Self::IntegrationUpdate),
            "INTEGRATION_DELETE" => Some(Self::IntegrationDelete),
            "GUILD_UPDATE" => Some(Self::GuildUpdate),
            "AUTOMOD_RULE_CREATE" => Some(Self::AutomodRuleCreate),
            "AUTOMOD_RULE_UPDATE" => Some(Self::AutomodRuleUpdate),
            "AUTOMOD_RULE_DELETE" => Some(Self::AutomodRuleDelete),
            "GUILD_EVENT_CREATE" => Some(Self::GuildEventCreate),
            "GUILD_EVENT_UPDATE" => Some(Self::GuildEventUpdate),
            "GUILD_EVENT_DELETE" => Some(Self::GuildEventDelete),
            "WEBHOOK_CREATE" => Some(Self::WebhookCreate),
            "WEBHOOK_UPDATE" => Some(Self::WebhookUpdate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Punishment {
    Ban,
    Kick,
    StripRoles,
    Timeout,
    LogOnly,
    None,
}

impl Punishment {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_uppercase().as_str() {
            "BAN" => Some(Self::Ban),
            "KICK" => Some(Self::Kick),
            "STRIP_ROLES" => Some(Self::StripRoles),
            "TIMEOUT" => Some(Self::Timeout),
            "LOG_ONLY" => Some(Self::LogOnly),
            "NONE" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ban => "BAN",
            Self::Kick => "KICK",
            Self::StripRoles => "STRIP_ROLES",
            Self::Timeout => "TIMEOUT",
            Self::LogOnly => "LOG_ONLY",
            Self::None => "NONE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentMatch {
    pub module: ActionType,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalModuleConfig {
    pub enabled: bool,
    pub threshold: u32,
    pub window_secs: u32,
    pub punishment: Punishment,
    pub log_only: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuildConfig {
    pub enabled: bool,
    pub modules: HashMap<u8, InternalModuleConfig>,
    pub log_channel_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ThreatResult {
    pub score: u32,
    pub triggered: bool,
    pub punishment: Punishment,
    pub reason: String,
    pub action: ActionType,
    pub should_restore: bool,
    pub count_in_window: u32,
}
