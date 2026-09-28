use crate::interaction::CommandCtx;
use hikari_database::{
    models::antinuke_config::AntinukeModuleConfigRow,
    repository::antinuke_repository::AntinukeRepository,
};
use hikari_utils::error::HikariError;
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};

#[derive(CommandOption, CreateOption, Debug, Clone)]
pub enum ActionChoice {
    #[option(name = "Banning Members", value = "BAN_ADD")]
    BanAdd,
    #[option(name = "Kicking Members", value = "MEMBER_KICK")]
    MemberKick,
    #[option(name = "Deleting Roles", value = "ROLE_DELETE")]
    RoleDelete,
    #[option(name = "Creating Roles", value = "ROLE_CREATE")]
    RoleCreate,
    #[option(name = "Deleting Channels", value = "CHANNEL_DELETE")]
    ChannelDelete,
    #[option(name = "Creating Channels", value = "CHANNEL_CREATE")]
    ChannelCreate,
}

impl ActionChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BanAdd => "BAN_ADD",
            Self::MemberKick => "MEMBER_KICK",
            Self::RoleDelete => "ROLE_DELETE",
            Self::RoleCreate => "ROLE_CREATE",
            Self::ChannelDelete => "CHANNEL_DELETE",
            Self::ChannelCreate => "CHANNEL_CREATE",
        }
    }

    pub fn from_value(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "MEMBER_KICK" => Self::MemberKick,
            "ROLE_DELETE" => Self::RoleDelete,
            "ROLE_CREATE" => Self::RoleCreate,
            "CHANNEL_DELETE" => Self::ChannelDelete,
            "CHANNEL_CREATE" => Self::ChannelCreate,
            _ => Self::BanAdd,
        }
    }
}

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(
    name = "limit",
    desc = "Set how many actions trigger a punishment (0 = instant)"
)]
pub struct SetLimit {
    #[command(desc = "The action type to configure")]
    pub action: ActionChoice,
    #[command(
        desc = "Max actions before punishment fires (0 = zero-tolerance)",
        min_value = 0,
        max_value = 20
    )]
    pub limit: i64,
}

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "set", desc = "Configure AntiNuke limits")]
pub enum AntinukeSet {
    #[command(name = "limit")]
    Limit(SetLimit),
}

#[tracing::instrument(skip_all, fields(guild = cx.guild_id, action = action.as_str(), limit))]
pub async fn execute(
    cx: &CommandCtx<'_>,
    action: &ActionChoice,
    limit: i64,
) -> Result<String, HikariError> {
    let repo = AntinukeRepository::new(cx.module.db.clone());
    let action_str = action.as_str();

    let punishment = repo
        .get_module_configs(cx.guild_id)
        .await?
        .into_iter()
        .find(|c| c.action_type == action_str)
        .map(|c| c.punishment)
        .unwrap_or_else(|| "BAN".into());

    repo.upsert_module_config(&AntinukeModuleConfigRow {
        guild_id: cx.guild_id,
        action_type: action_str.into(),
        enabled: true,
        threshold: limit as i32,
        window_secs: 60,
        punishment: punishment.clone(),
        log_only: false,
    })
    .await?;

    hikari_antinuke::reload_guild_config(&cx.module.db, cx.guild_id as u64).await;

    if limit == 0 {
        Ok(format!(
            "**zero-tolerance** for `{action_str}` — first detection → instant **{punishment}**."
        ))
    } else {
        Ok(format!(
            "limit for `{action_str}`: max **{limit}** actions → **{punishment}**."
        ))
    }
}
