use twilight_model::channel::message::Embed;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle, Component};
use twilight_util::builder::embed::EmbedBuilder;

pub fn build_stylish_embed(_title: &str, description: &str, color: u32) -> Embed {
    EmbedBuilder::new()
        .description(description)
        .color(color)
        .build()
}

pub fn build_antinuke_log_buttons(user_id: u64) -> Component {
    let whitelist_btn = Button {
        custom_id: Some(format!("antinuke_wl:{}", user_id)),
        disabled: false,
        emoji: None,
        label: Some("Whitelist".to_string()),
        style: ButtonStyle::Secondary,
        url: None,
        sku_id: None,
        id: None,
    };

    let unban_btn = Button {
        custom_id: Some(format!("antinuke_unban:{}", user_id)),
        disabled: false,
        emoji: None,
        label: Some("Unban".to_string()),
        style: ButtonStyle::Danger,
        url: None,
        sku_id: None,
        id: None,
    };

    Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(whitelist_btn),
            Component::Button(unban_btn),
        ],
        id: None,
    })
}

pub fn build_antinuke_settings_buttons(enabled: bool) -> Component {
    let toggle_btn = Button {
        custom_id: Some("antinuke_toggle".to_string()),
        disabled: false,
        emoji: None,
        label: Some(if enabled {
            "Disable".to_string()
        } else {
            "Enable".to_string()
        }),
        style: if enabled {
            ButtonStyle::Danger
        } else {
            ButtonStyle::Success
        },
        url: None,
        sku_id: None,
        id: None,
    };

    Component::ActionRow(ActionRow {
        components: vec![Component::Button(toggle_btn)],
        id: None,
    })
}

pub fn build_automod_settings_buttons(spam: bool, antilink: bool, ghostping: bool) -> Component {
    let spam_btn = Button {
        custom_id: Some("automod_toggle:spam".to_string()),
        disabled: false,
        emoji: None,
        label: Some(if spam {
            "Spam: On".to_string()
        } else {
            "Spam: Off".to_string()
        }),
        style: if spam {
            ButtonStyle::Success
        } else {
            ButtonStyle::Danger
        },
        url: None,
        sku_id: None,
        id: None,
    };

    let antilink_btn = Button {
        custom_id: Some("automod_toggle:antilink".to_string()),
        disabled: false,
        emoji: None,
        label: Some(if antilink {
            "Anti-Link: On".to_string()
        } else {
            "Anti-Link: Off".to_string()
        }),
        style: if antilink {
            ButtonStyle::Success
        } else {
            ButtonStyle::Danger
        },
        url: None,
        sku_id: None,
        id: None,
    };

    let ghostping_btn = Button {
        custom_id: Some("automod_toggle:ghostping".to_string()),
        disabled: false,
        emoji: None,
        label: Some(if ghostping {
            "GhostPing: On".to_string()
        } else {
            "GhostPing: Off".to_string()
        }),
        style: if ghostping {
            ButtonStyle::Success
        } else {
            ButtonStyle::Danger
        },
        url: None,
        sku_id: None,
        id: None,
    };

    Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(spam_btn),
            Component::Button(antilink_btn),
            Component::Button(ghostping_btn),
        ],
        id: None,
    })
}
