use crate::models::types::{ActionType, ContentMatch, InternalModuleConfig};
use memchr::memmem;
use std::collections::HashMap;

pub struct ContentScanInput<'a> {
    pub content: &'a str,
    pub author_roles: &'a [u64],
}

#[must_use]
pub fn scan(
    input: &ContentScanInput<'_>,
    modules: &HashMap<u8, InternalModuleConfig>,
) -> Vec<ContentMatch> {
    let mut matches = Vec::with_capacity(2);
    let bytes = input.content.as_bytes();

    if let Some(cfg) = modules.get(&(ActionType::EveryonePing as u8))
        && cfg.enabled
        && memmem::find(bytes, b"@everyone").is_some()
    {
        matches.push(ContentMatch {
            module: ActionType::EveryonePing,
            detail: "@everyone ping detected".to_owned(),
        });
    }

    if let Some(cfg) = modules.get(&(ActionType::HerePing as u8))
        && cfg.enabled
        && memmem::find(bytes, b"@here").is_some()
    {
        matches.push(ContentMatch {
            module: ActionType::HerePing,
            detail: "@here ping detected".to_owned(),
        });
    }

    if let Some(cfg) = modules.get(&(ActionType::LinkInMessage as u8))
        && cfg.enabled
        && (memmem::find(bytes, b"http://").is_some() || memmem::find(bytes, b"https://").is_some())
    {
        matches.push(ContentMatch {
            module: ActionType::LinkInMessage,
            detail: "Link detected in message".to_owned(),
        });
    }

    if let Some(cfg) = modules.get(&(ActionType::RolePing as u8))
        && cfg.enabled
        && memmem::find(bytes, b"<@&").is_some()
    {
        matches.push(ContentMatch {
            module: ActionType::RolePing,
            detail: "Role ping detected".to_owned(),
        });
    }

    matches
}
