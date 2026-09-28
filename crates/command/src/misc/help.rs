use crate::core::handler::ModuleCommand;
use crate::{interaction::InteractionContext, prefix::PrefixContext};
use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle, Component};

#[derive(CommandModel, CreateCommand, Debug, Clone)]
#[command(name = "help", desc = "Show hikari help menu")]
pub struct HelpCommand {}

fn build_help_buttons() -> Component {
    let home_btn = Button {
        custom_id: Some("help:home".to_string()),
        disabled: false,
        emoji: None,
        label: Some("Home".to_string()),
        style: ButtonStyle::Danger,
        url: None,
        sku_id: None,
        id: None,
    };

    let antinuke_btn = Button {
        custom_id: Some("help:antinuke".to_string()),
        disabled: false,
        emoji: None,
        label: Some("AntiNuke".to_string()),
        style: ButtonStyle::Secondary,
        url: None,
        sku_id: None,
        id: None,
    };

    let automod_btn = Button {
        custom_id: Some("help:automod".to_string()),
        disabled: false,
        emoji: None,
        label: Some("AutoMod".to_string()),
        style: ButtonStyle::Secondary,
        url: None,
        sku_id: None,
        id: None,
    };

    let premium_btn = Button {
        custom_id: Some("help:premium".to_string()),
        disabled: false,
        emoji: None,
        label: Some("Premium".to_string()),
        style: ButtonStyle::Secondary,
        url: None,
        sku_id: None,
        id: None,
    };

    Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(home_btn),
            Component::Button(antinuke_btn),
            Component::Button(automod_btn),
            Component::Button(premium_btn),
        ],
        id: None,
    })
}

pub struct HelpHandler;

#[async_trait]
impl ModuleCommand for HelpHandler {
    fn name(&self) -> &'static str {
        "help"
    }

    fn command(&self) -> twilight_model::application::command::Command {
        HelpCommand::create_command().into()
    }

    async fn handle_slash(
        &self,
        ctx: InteractionContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        let text =
            "a fast and minimal protection bot.\nselect a category below to explore commands.";
        let v2_components = vec![Component::Container(
            twilight_model::channel::message::component::Container {
                id: None,
                accent_color: Some(Some(module.embed_color)),
                spoiler: None,
                components: vec![
                    Component::Section(twilight_model::channel::message::component::Section {
                        id: None,
                        components: vec![Component::TextDisplay(
                            twilight_model::channel::message::component::TextDisplay {
                                id: None,
                                content: format!("**hikari**\n{}", text),
                            },
                        )],
                        accessory: Box::new(Component::Button(
                            twilight_model::channel::message::component::Button {
                                id: None,
                                style: ButtonStyle::Secondary,
                                disabled: false,
                                label: Some("Settings".to_string()),
                                emoji: None,
                                custom_id: Some("help:settings".to_string()),
                                url: None,
                                sku_id: None,
                            },
                        )),
                    }),
                    build_help_buttons(),
                ],
            },
        )];

        let builder = twilight_util::builder::InteractionResponseDataBuilder::new()
            .flags(twilight_model::channel::message::MessageFlags::IS_COMPONENTS_V2)
            .components(v2_components);

        let ic = module.discord.interaction(ctx.interaction.application_id);
        ic.create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &twilight_model::http::interaction::InteractionResponse {
                kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
                data: Some(builder.build()),
            },
        ).await?;
        Ok(())
    }

    async fn handle_prefix(
        &self,
        ctx: &PrefixContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError> {
        let text =
            "a fast and minimal protection bot.\nselect a category below to explore commands.";
        let v2_components = vec![Component::Container(
            twilight_model::channel::message::component::Container {
                id: None,
                accent_color: Some(Some(module.embed_color)),
                spoiler: None,
                components: vec![
                    Component::Section(twilight_model::channel::message::component::Section {
                        id: None,
                        components: vec![Component::TextDisplay(
                            twilight_model::channel::message::component::TextDisplay {
                                id: None,
                                content: format!("**hikari**\n{}", text),
                            },
                        )],
                        accessory: Box::new(Component::Button(
                            twilight_model::channel::message::component::Button {
                                id: None,
                                style: ButtonStyle::Secondary,
                                disabled: false,
                                label: Some("Settings".to_string()),
                                emoji: None,
                                custom_id: Some("help:settings".to_string()),
                                url: None,
                                sku_id: None,
                            },
                        )),
                    }),
                    build_help_buttons(),
                ],
            },
        )];

        module
            .discord
            .create_message(ctx.message.channel_id)
            .flags(twilight_model::channel::message::MessageFlags::IS_COMPONENTS_V2)
            .components(&v2_components)
            .await?;
        Ok(())
    }

    async fn handle_component(
        &self,
        ctx: &InteractionContext,
        custom_id: &str,
        module: &ModuleContext,
    ) -> Result<bool, HikariError> {
        if let Some(category) = custom_id.strip_prefix("help:") {
            let (title, text) = match category {
                "antinuke" => (
                    "AntiNuke",
                    "protect your server from rogue admins and malicious attacks.\n\n• `enable` · `disable` · `settings` · `whitelisted`\n• `set limit <action> <count>`\n• `punishment <action> <type>`\n• `whitelist add @user` · `whitelist remove @user`",
                ),
                "automod" => (
                    "AutoMod",
                    "automatically block spam and malicious links.\n\n• `enable` · `disable`",
                ),
                "premium" => (
                    "Premium",
                    "unlock advanced capabilities for your server.\n\n• `add` · `revoke` · `status` · `activate` · `noprefix`",
                ),
                _ => (
                    "hikari",
                    "a fast and minimal protection bot.\nselect a category below to explore commands.",
                ),
            };

            let v2_components = vec![Component::Container(
                twilight_model::channel::message::component::Container {
                    id: None,
                    accent_color: Some(Some(module.embed_color)),
                    spoiler: None,
                    components: vec![
                        Component::Section(twilight_model::channel::message::component::Section {
                            id: None,
                            components: vec![Component::TextDisplay(
                                twilight_model::channel::message::component::TextDisplay {
                                    id: None,
                                    content: format!("**{}**\n{}", title, text),
                                },
                            )],
                            accessory: Box::new(Component::Button(
                                twilight_model::channel::message::component::Button {
                                    id: None,
                                    style: ButtonStyle::Secondary,
                                    disabled: false,
                                    label: Some("Settings".to_string()),
                                    emoji: None,
                                    custom_id: Some("help:settings".to_string()),
                                    url: None,
                                    sku_id: None,
                                },
                            )),
                        }),
                        build_help_buttons(),
                    ],
                },
            )];

            let ic = module.discord.interaction(ctx.interaction.application_id);
            ic.create_response(
                ctx.interaction.id,
                &ctx.interaction.token,
                &twilight_model::http::interaction::InteractionResponse {
                    kind: twilight_model::http::interaction::InteractionResponseType::UpdateMessage,
                    data: Some(
                        twilight_util::builder::InteractionResponseDataBuilder::new()
                            .flags(twilight_model::channel::message::MessageFlags::IS_COMPONENTS_V2)
                            .components(v2_components)
                            .build(),
                    ),
                },
            )
            .await?;

            return Ok(true);
        }

        Ok(false)
    }
}
