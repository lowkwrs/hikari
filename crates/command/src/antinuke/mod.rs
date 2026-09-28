pub mod disable;
pub mod enable;
pub mod punishment;
pub mod set_limit;
pub mod settings;
pub mod whitelist_add;
pub mod whitelist_remove;
pub mod whitelisted;

use crate::core::handler::ModuleCommand;
use crate::{
    interaction::{CommandCtx, InteractionContext},
    prefix::PrefixContext,
};
use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use punishment::{AntinukePunishment, PunishmentChoice};
use set_limit::{ActionChoice, AntinukeSet, SetLimit};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::InteractionData,
    guild::Permissions,
    id::{Id, marker::UserMarker},
};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "whitelist", desc = "Manage users exempt from AntiNuke")]
pub enum AntinukeWhitelistGroup {
    #[command(name = "add")]
    Add(whitelist_add::WhitelistAdd),
    #[command(name = "remove")]
    Remove(whitelist_remove::WhitelistRemove),
}

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "antinuke", desc = "Configure the hikari AntiNuke engine")]
pub enum AntinukeCommand {
    #[command(name = "enable")]
    Enable(enable::AntinukeEnable),
    #[command(name = "disable")]
    Disable(disable::AntinukeDisable),
    #[command(name = "settings")]
    Settings(settings::AntinukeSettings),
    #[command(name = "whitelisted")]
    Whitelisted(whitelisted::AntinukeWhitelisted),
    #[command(name = "set")]
    Set(set_limit::AntinukeSet),
    #[command(name = "punishment")]
    Punishment(punishment::AntinukePunishment),
    #[command(name = "whitelist")]
    Whitelist(AntinukeWhitelistGroup),
}

impl AntinukeCommand {
    pub async fn run(&self, cx: &CommandCtx<'_>) -> Result<String, HikariError> {
        match self {
            Self::Enable(_) => enable::execute(cx).await,
            Self::Disable(_) => disable::execute(cx).await,
            Self::Settings(_) => settings::execute(cx).await,
            Self::Whitelisted(_) => whitelisted::execute(cx).await,
            Self::Set(AntinukeSet::Limit(l)) => set_limit::execute(cx, &l.action, l.limit).await,
            Self::Punishment(p) => punishment::execute(cx, &p.action, &p.punishment).await,
            Self::Whitelist(AntinukeWhitelistGroup::Add(a)) => {
                whitelist_add::execute(cx, a.user.get() as i64).await
            }
            Self::Whitelist(AntinukeWhitelistGroup::Remove(r)) => {
                whitelist_remove::execute(cx, r.user.get() as i64).await
            }
        }
    }
}

const PERM_DENIED: &str = "missing permissions. you need **manage guild** or **admin**.";

#[inline]
fn check_perms(ctx: &InteractionContext) -> bool {
    ctx.has_permission(Permissions::MANAGE_GUILD) || ctx.has_permission(Permissions::ADMINISTRATOR)
}

pub async fn handle_slash(
    interaction_ctx: InteractionContext,
    module: &ModuleContext,
) -> Result<(), HikariError> {
    let guild_id = interaction_ctx
        .guild_id()
        .ok_or_else(|| HikariError::Internal("Guild only".into()))?;
    let user_id = interaction_ctx
        .author_id()
        .ok_or_else(|| HikariError::Internal("No author".into()))?;

    if !check_perms(&interaction_ctx) {
        let embed = hikari_embeds::ui::build_stylish_embed(
            "Permission Denied",
            PERM_DENIED,
            module.embed_color,
        );
        return interaction_ctx.respond_embed(module, embed, None).await;
    }

    let cmd = match &interaction_ctx.interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => {
            AntinukeCommand::from_interaction((**data).clone().into())
                .map_err(|e| HikariError::Internal(format!("Parse error: {e}")))?
        }
        _ => return Err(HikariError::Internal("Not a command".into())),
    };

    let is_settings = matches!(cmd, AntinukeCommand::Settings(_));

    if !is_settings {
        interaction_ctx.defer(module).await?;
    }

    let cx = CommandCtx::new(guild_id.get() as i64, user_id.get() as i64, module);
    let reply = cmd.run(&cx).await?;

    let action_row = if is_settings {
        let repo = hikari_database::repository::antinuke_repository::AntinukeRepository::new(
            module.db.clone(),
        );
        let enabled = repo
            .get_config(cx.guild_id)
            .await?
            .map(|c| c.enabled)
            .unwrap_or(false);
        vec![hikari_embeds::ui::build_antinuke_settings_buttons(enabled)]
    } else {
        vec![]
    };

    let embed = hikari_embeds::ui::build_stylish_embed(
        if is_settings {
            "AntiNuke Settings"
        } else {
            "AntiNuke"
        },
        &reply,
        module.embed_color,
    );

    interaction_ctx
        .respond_embed(module, embed, Some(action_row))
        .await
}

const HELP_TEXT: &str = "**Sub-commands:**\n\
    `enable` · `disable` · `settings` · `whitelisted`\n\
    `set limit <action> <count>` · `punishment <action> <type>`\n\
    `whitelist add @user` · `whitelist remove @user`\n\n\
    **Actions:** `BAN_ADD` `MEMBER_KICK` `ROLE_DELETE` `ROLE_CREATE` `CHANNEL_DELETE` `CHANNEL_CREATE`\n\
    **Punishments:** `BAN` `KICK` `STRIP_ROLES`";

pub async fn handle_prefix(
    prefix_ctx: &PrefixContext,
    module: &ModuleContext,
) -> Result<(), HikariError> {
    let args = &prefix_ctx.args;

    if args.is_empty() {
        let embed =
            hikari_embeds::ui::build_stylish_embed("AntiNuke Help", HELP_TEXT, module.embed_color);
        return prefix_ctx.reply_with_ui(embed, vec![]).await;
    }

    let Some(cmd) = parse_prefix(args) else {
        let embed = hikari_embeds::ui::build_stylish_embed(
            "AntiNuke",
            "Unknown sub-command. Run without arguments to see help.",
            module.embed_color,
        );
        return prefix_ctx.reply_with_ui(embed, vec![]).await;
    };

    let cx = CommandCtx::new(
        prefix_ctx.guild_id.get() as i64,
        prefix_ctx.message.author.id.get() as i64,
        module,
    );
    let reply = cmd.run(&cx).await?;
    let embed = hikari_embeds::ui::build_stylish_embed("AntiNuke", &reply, module.embed_color);
    prefix_ctx.reply_with_ui(embed, vec![]).await
}

fn parse_prefix(args: &[String]) -> Option<AntinukeCommand> {
    match args[0].to_lowercase().as_str() {
        "enable" => Some(AntinukeCommand::Enable(enable::AntinukeEnable {})),
        "disable" => Some(AntinukeCommand::Disable(disable::AntinukeDisable {})),
        "settings" => Some(AntinukeCommand::Settings(settings::AntinukeSettings {})),
        "whitelisted" => Some(AntinukeCommand::Whitelisted(
            whitelisted::AntinukeWhitelisted {},
        )),
        "set" if args.get(1).map(|s| s.as_str()) == Some("limit") => {
            let action = ActionChoice::from_value(args.get(2).map(String::as_str).unwrap_or(""));
            let limit = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(AntinukeCommand::Set(AntinukeSet::Limit(SetLimit {
                action,
                limit,
            })))
        }
        "punishment" => {
            let action = ActionChoice::from_value(args.get(1).map(String::as_str).unwrap_or(""));
            let pun = PunishmentChoice::from_value(args.get(2).map(String::as_str).unwrap_or(""));
            Some(AntinukeCommand::Punishment(AntinukePunishment {
                action,
                punishment: pun,
            }))
        }
        "whitelist" => match args.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("add") => Some(AntinukeCommand::Whitelist(AntinukeWhitelistGroup::Add(
                whitelist_add::WhitelistAdd {
                    user: parse_uid(args.get(2))?,
                },
            ))),
            Some("remove") => Some(AntinukeCommand::Whitelist(AntinukeWhitelistGroup::Remove(
                whitelist_remove::WhitelistRemove {
                    user: parse_uid(args.get(2))?,
                },
            ))),
            _ => None,
        },
        _ => None,
    }
}

#[inline]
fn parse_uid(raw: Option<&String>) -> Option<Id<UserMarker>> {
    raw.and_then(|s| {
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse::<u64>().ok().map(Id::new)
    })
}

pub async fn settings_text(guild_id: i64, module: &ModuleContext) -> Result<String, HikariError> {
    let cx = CommandCtx::new(guild_id, 0, module);
    settings::execute(&cx).await
}

pub struct AntinukeHandler;

#[async_trait]
impl ModuleCommand for AntinukeHandler {
    fn name(&self) -> &'static str {
        "antinuke"
    }

    fn command(&self) -> twilight_model::application::command::Command {
        AntinukeCommand::create_command().into()
    }

    async fn handle_slash(
        &self,
        ctx: InteractionContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        handle_slash(ctx, module).await
    }

    async fn handle_prefix(
        &self,
        ctx: &PrefixContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        handle_prefix(ctx, module).await
    }

    async fn handle_component(
        &self,
        interaction_ctx: &InteractionContext,
        custom_id: &str,
        module: &ModuleContext,
    ) -> Result<bool, HikariError> {
        let guild_id = interaction_ctx
            .guild_id()
            .ok_or_else(|| HikariError::Internal("Guild only".into()))?;

        if !custom_id.starts_with("antinuke_") {
            return Ok(false);
        }

        if !check_perms(interaction_ctx) {
            interaction_ctx
                .respond_ephemeral(module, "missing permissions. you need **manage guild**.")
                .await?;
            return Ok(true);
        }

        if custom_id.starts_with("antinuke_wl:") {
            interaction_ctx.defer(module).await?;

            let target_id: u64 = custom_id
                .trim_start_matches("antinuke_wl:")
                .parse()
                .map_err(|_| HikariError::Internal("Invalid ID".into()))?;
            let clicker_id = interaction_ctx
                .author_id()
                .ok_or_else(|| HikariError::Internal("No author".into()))?;

            let repo = hikari_database::repository::antinuke_repository::AntinukeRepository::new(
                module.db.clone(),
            );
            repo.add_whitelist(
                guild_id.get() as i64,
                target_id as i64,
                clicker_id.get() as i64,
            )
            .await?;
            hikari_antinuke::whitelist_add(guild_id.get(), target_id).await;

            let embed = hikari_embeds::ui::build_stylish_embed(
                "Whitelist",
                &format!("<@{}> is now whitelisted.", target_id),
                module.embed_color,
            );
            interaction_ctx.respond_embed(module, embed, None).await?;
        } else if custom_id.starts_with("antinuke_unban:") {
            interaction_ctx.defer(module).await?;

            let target_id: u64 = custom_id
                .trim_start_matches("antinuke_unban:")
                .parse()
                .map_err(|_| HikariError::Internal("Invalid ID".into()))?;

            let content = match module
                .discord
                .delete_ban(guild_id, Id::new(target_id))
                .await
            {
                Ok(_) => format!("<@{}> unbanned.", target_id),
                Err(e) => format!("failed: {}", e),
            };

            let embed =
                hikari_embeds::ui::build_stylish_embed("Unban", &content, module.embed_color);
            interaction_ctx.respond_embed(module, embed, None).await?;
        } else if custom_id == "antinuke_toggle" {
            interaction_ctx.defer_update(module).await?;

            let repo = hikari_database::repository::antinuke_repository::AntinukeRepository::new(
                module.db.clone(),
            );
            let mut config = repo
                .get_config(guild_id.get() as i64)
                .await?
                .unwrap_or_else(
                    || hikari_database::models::antinuke_config::AntinukeConfig {
                        guild_id: guild_id.get() as i64,
                        enabled: false,
                        log_channel_id: None,
                        updated_at: chrono::Utc::now(),
                    },
                );

            config.enabled = !config.enabled;

            if config.enabled && config.log_channel_id.is_none() {
                let channel = module
                    .discord
                    .create_guild_channel(guild_id, "hikari-logs")
                    .await?
                    .model()
                    .await?;
                config.log_channel_id = Some(channel.id.get() as i64);
            }

            let new_state = config.enabled;
            repo.upsert_config(&config).await?;
            hikari_antinuke::reload_guild_config(&module.db, guild_id.get()).await;

            let settings_str = settings_text(guild_id.get() as i64, module).await?;
            let embed = hikari_embeds::ui::build_stylish_embed(
                "AntiNuke Settings",
                &settings_str,
                module.embed_color,
            );
            let action_row = hikari_embeds::ui::build_antinuke_settings_buttons(new_state);

            interaction_ctx
                .respond_embed(module, embed, Some(vec![action_row]))
                .await?;
        }

        Ok(true)
    }
}
