use super::set_limit::ActionChoice;
use crate::interaction::CommandCtx;
use hikari_database::{
    models::antinuke_config::AntinukeModuleConfigRow,
    repository::antinuke_repository::AntinukeRepository,
};
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};

#[derive(CommandOption, CreateOption, Debug, Clone)]
pub enum PunishmentChoice {
    #[option(name = "Ban", value = "BAN")]
    Ban,
    #[option(name = "Kick", value = "KICK")]
    Kick,
    #[option(name = "Strip Roles", value = "STRIP_ROLES")]
    StripRoles,
}

impl PunishmentChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ban => "BAN",
            Self::Kick => "KICK",
            Self::StripRoles => "STRIP_ROLES",
        }
    }

    pub fn from_value(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "KICK" => Self::Kick,
            "STRIP_ROLES" => Self::StripRoles,
            _ => Self::Ban,
        }
    }
}

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "punishment",
    desc = "Set the punishment applied when a limit is exceeded"
)]
pub struct AntinukePunishment {
    #[command(desc = "The action type to configure")]
    pub action: ActionChoice,
    #[command(desc = "Punishment to apply when the threshold is reached")]
    pub punishment: PunishmentChoice,
}

#[tracing::instrument(skip_all, fields(guild = cx.guild_id, action = action.as_str(), punishment = punishment.as_str()))]
pub async fn execute(
    cx: &CommandCtx<'_>,
    action: &ActionChoice,
    punishment: &PunishmentChoice,
) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());
    let action_str = action.as_str();
    let punishment_str = punishment.as_str();

    let mut row = repo
        .get_module_configs(cx.guild_id)
        .await?
        .into_iter()
        .find(|c| c.action_type == action_str)
        .unwrap_or_else(|| AntinukeModuleConfigRow {
            guild_id: cx.guild_id,
            action_type: action_str.into(),
            enabled: false,
            threshold: 3,
            window_secs: 60,
            punishment: "BAN".into(),
            log_only: false,
        });

    row.punishment = punishment_str.into();
    repo.upsert_module_config(&row).await?;
    hikari_antinuke::reload_guild_config(&cx.module.db, cx.guild_id as u64).await;

    Ok(format!(
        "punishment for `{action_str}` set to **{punishment_str}**."
    ))
}
