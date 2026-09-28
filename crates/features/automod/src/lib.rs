use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::{
    guild::auto_moderation::{AutoModerationEventType, AutoModerationKeywordPresetType},
    id::{Id, marker::GuildMarker},
};

static KEYWORD_LIST: &[&str] = &[
    "*https://*",
    "*discord.gg*",
    "*discord.com/invite*",
    "anal",
    "anus*",
    "arse",
    "asshat*",
    "asshole*",
    "b1tch*",
    "ballsack*",
    "bdsm*",
    "beastiality*",
    "biatch*",
    "bitch*",
    "blowjob*",
    "bollock*",
    "boner*",
    "boob",
    "boobs",
    "buttplug*",
    "clitoris*",
    "cock",
    "cum",
    "cunt*",
    "deepthroat*",
    "dick*",
    "dildo",
    "ejaculate",
    "erection*",
    "feck*",
    "felching*",
    "fellatio*",
    "fuck*",
    "fucks*",
    "genitals*",
    "horny*",
    "kys*",
    "labia*",
    "masturbate*",
    "nakedphotos*",
    "p0rn",
    "penis",
    "piss",
    "porn",
    "pornhub*",
    "porno*",
    "pussy*",
    "rape",
    "rimjob*",
    "rule34*",
    "scrotum*",
    "sexslaves*",
    "sh1t",
    "shemale*",
    "smegma*",
    "sperm",
    "spunk*",
    "strap-on*",
    "strapon*",
    "stripper*",
    "testicle*",
    "tits*",
    "titty*",
    "tosser*",
    "trannie*",
    "tranny*",
    "tubgirl*",
    "twat*",
    "vagina*",
    "wank*",
    "whore*",
    "x-rated*",
    "zoophile*",
];

/// Enable 4 AutoMod protection rules for a guild.
/// Returns the number of rules successfully created.
#[tracing::instrument(skip(http), fields(guild = guild_id))]
pub async fn enable_rules(guild_id: u64, http: &Arc<HttpClient>) -> usize {
    let guild: Id<GuildMarker> = Id::new(guild_id);
    let mut created = 0usize;

    // First fetch existing rules to avoid MAX_RULES_OF_TYPE_EXCEEDED
    if let Ok(res) = http.auto_moderation_rules(guild).await
        && let Ok(rules) = res.model().await
    {
        for rule in rules {
            let _ = http.delete_auto_moderation_rule(guild, rule.id).await;
        }
    }

    match http
        .create_auto_moderation_rule(
            guild,
            "hikari | Keyword Filter",
            AutoModerationEventType::MessageSend,
        )
        .enabled(true)
        .action_block_message()
        .with_keyword(KEYWORD_LIST, &[], &[])
        .await
    {
        Ok(_) => created += 1,
        Err(e) => tracing::error!("Failed to create Keyword Filter: {:?}", e),
    }

    match http
        .create_auto_moderation_rule(
            guild,
            "hikari | Mention Spam",
            AutoModerationEventType::MessageSend,
        )
        .enabled(true)
        .action_block_message()
        .with_mention_spam(5)
        .await
    {
        Ok(_) => created += 1,
        Err(e) => tracing::error!("Failed to create Mention Spam: {:?}", e),
    }

    let presets = [
        AutoModerationKeywordPresetType::Profanity,
        AutoModerationKeywordPresetType::SexualContent,
        AutoModerationKeywordPresetType::Slurs,
    ];

    match http
        .create_auto_moderation_rule(
            guild,
            "hikari | Content Filter",
            AutoModerationEventType::MessageSend,
        )
        .enabled(true)
        .action_block_message()
        .with_keyword_preset(&presets, &[])
        .await
    {
        Ok(_) => created += 1,
        Err(e) => tracing::error!("Failed to create Content Filter: {:?}", e),
    }

    match http
        .create_auto_moderation_rule(
            guild,
            "hikari | Spam Shield",
            AutoModerationEventType::MessageSend,
        )
        .enabled(true)
        .action_block_message()
        .with_spam()
        .await
    {
        Ok(_) => created += 1,
        Err(e) => tracing::error!("Failed to create Spam Shield: {:?}", e),
    }

    tracing::info!(guild = guild_id, rules = created, "AutoMod rules enabled");
    created
}

#[tracing::instrument(skip(http), fields(guild = guild_id))]
pub async fn disable_rules(guild_id: u64, http: &Arc<HttpClient>) -> usize {
    let guild: Id<GuildMarker> = Id::new(guild_id);
    let mut deleted = 0usize;

    if let Ok(res) = http.auto_moderation_rules(guild).await
        && let Ok(rules) = res.model().await
    {
        for rule in rules {
            if http
                .delete_auto_moderation_rule(guild, rule.id)
                .await
                .is_ok()
            {
                deleted += 1;
            }
        }
    }

    tracing::info!(guild = guild_id, rules = deleted, "AutoMod rules disabled");
    deleted
}
