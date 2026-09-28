use crate::models::types::{ActionType, Punishment};

#[must_use]
pub fn base_score(action: ActionType) -> u32 {
    match action {
        ActionType::MemberPrune => 50,
        ActionType::GuildUpdate => 40,
        ActionType::BotAdd => 40,
        ActionType::BanAdd => 35,
        ActionType::ChannelDelete => 30,
        ActionType::RoleDelete => 30,
        ActionType::MemberKick => 25,
        ActionType::WebhookCreate => 25,
        ActionType::LinkInMessage => 25,
        ActionType::IntegrationCreate => 25,
        ActionType::RoleUpdate => 25,
        ActionType::AutomodRuleDelete => 20,
        ActionType::EveryonePing => 20,
        ActionType::IntegrationDelete => 20,
        ActionType::RoleCreate => 15,
        ActionType::ChannelCreate => 15,
        ActionType::EmojiDelete => 15,
        ActionType::StickerDelete => 15,
        ActionType::RolePing => 15,
        ActionType::IntegrationUpdate => 15,
        ActionType::WebhookUpdate => 15,
        ActionType::HerePing => 15,
        ActionType::BanRemove => 10,
        ActionType::MemberUpdate => 10,
        ActionType::AutomodRuleCreate => 10,
        ActionType::AutomodRuleUpdate => 10,
        ActionType::GuildEventDelete => 10,
        ActionType::ChannelUpdate => 10,
        ActionType::EmojiCreate => 8,
        ActionType::StickerCreate => 8,
        ActionType::EmojiUpdate => 5,
        ActionType::StickerUpdate => 5,
        ActionType::GuildEventCreate => 5,
        ActionType::GuildEventUpdate => 5,
    }
}

#[must_use]
pub fn burst_multiplier(count: usize) -> f32 {
    match count {
        0 | 1 => 0.5,
        2 => 0.8,
        3 => 1.0,
        4 => 1.4,
        5 => 1.8,
        6..=8 => 2.2,
        9..=12 => 2.8,
        _ => 3.5,
    }
}

#[must_use]
pub fn compute_score(action: ActionType, count: usize) -> u32 {
    let base = base_score(action) as f32;
    let mult = burst_multiplier(count);
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let score = (base * mult) as u32;
    score.min(100)
}

#[must_use]
pub fn resolve_punishment(
    punishment: Punishment,
    action: ActionType,
    count: u32,
    score: u32,
) -> (Punishment, String) {
    if punishment == Punishment::LogOnly {
        return (
            Punishment::LogOnly,
            format!(
                "{} detected {} times (score {})",
                action.as_str(),
                count,
                score
            ),
        );
    }
    let reason = format!(
        "[hikari AntiNuke] {} — {} actions detected (score: {})",
        action.as_str(),
        count,
        score
    );
    (punishment, reason)
}

#[must_use]
pub fn has_dangerous_perm_grant(old_perms: u64, new_perms: u64) -> bool {
    const DANGEROUS: u64 =
        (1 << 3) | (1 << 1) | (1 << 2) | (1 << 5) | (1 << 28) | (1 << 4) | (1 << 29) | (1 << 13);
    let added = new_perms & !old_perms;
    added & DANGEROUS != 0
}
