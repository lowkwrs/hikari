use hikari_utils::{error::HikariError, module::ModuleContext};
use std::sync::Arc;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    application::interaction::{
        Interaction, InteractionData, application_command::CommandDataOption,
    },
    channel::message::{Embed, component::Component},
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{
        Id,
        marker::{ChannelMarker, GuildMarker, UserMarker},
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub struct CommandCtx<'a> {
    pub guild_id: i64,
    pub author_id: i64,
    pub module: &'a ModuleContext,
}

impl<'a> CommandCtx<'a> {
    #[inline]
    pub fn new(guild_id: i64, author_id: i64, module: &'a ModuleContext) -> Self {
        Self {
            guild_id,
            author_id,
            module,
        }
    }
}

pub struct InteractionContext {
    pub interaction: Arc<Interaction>,
    responded: std::sync::atomic::AtomicBool,
}

impl InteractionContext {
    #[inline]
    pub fn new(interaction: Interaction) -> Self {
        Self {
            interaction: Arc::new(interaction),
            responded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.interaction.guild_id
    }

    #[inline]
    pub fn author_id(&self) -> Option<Id<UserMarker>> {
        self.interaction.author_id()
    }

    #[inline]
    pub fn channel_id(&self) -> Option<Id<ChannelMarker>> {
        self.interaction.channel.as_ref().map(|c| c.id)
    }

    #[inline]
    pub fn has_permission(&self, perm: Permissions) -> bool {
        self.interaction
            .member
            .as_ref()
            .and_then(|m| m.permissions)
            .map(|p| p.contains(perm))
            .unwrap_or(false)
    }

    #[inline]
    pub fn extract_command_name(&self) -> Result<&str, HikariError> {
        match &self.interaction.data {
            Some(InteractionData::ApplicationCommand(cmd)) => Ok(&cmd.name),
            _ => Err(HikariError::Internal("No command data".into())),
        }
    }

    #[inline]
    pub fn extract_custom_id(&self) -> Result<&str, HikariError> {
        match &self.interaction.data {
            Some(InteractionData::MessageComponent(data)) => Ok(&data.custom_id),
            _ => Err(HikariError::Internal("Not a component".into())),
        }
    }

    #[inline]
    pub fn is_component(&self) -> bool {
        matches!(
            &self.interaction.data,
            Some(InteractionData::MessageComponent(_))
        )
    }

    #[inline]
    pub fn extract_subcommand(&self) -> Result<&str, HikariError> {
        if let Some(InteractionData::ApplicationCommand(cmd)) = &self.interaction.data
            && let Some(opt) = cmd.options.first()
        {
            return Ok(&opt.name);
        }
        Err(HikariError::Internal("No subcommand found".to_string()))
    }

    #[inline]
    pub fn extract_subcommand_options(&self) -> Result<&[CommandDataOption], HikariError> {
        if let Some(InteractionData::ApplicationCommand(cmd)) = &self.interaction.data
            && let Some(opt) = cmd.options.first()
            && let CommandOptionValue::SubCommand(opts) = &opt.value
        {
            return Ok(opts);
        }
        Err(HikariError::Internal(
            "No subcommand options found".to_string(),
        ))
    }

    #[inline]
    pub async fn respond(&self, module: &ModuleContext, content: &str) -> Result<(), HikariError> {
        self.respond_ephemeral(module, content).await
    }

    #[inline]
    pub async fn defer(&self, module: &ModuleContext) -> Result<(), HikariError> {
        if self
            .responded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(());
        }

        module
            .discord
            .interaction(self.interaction.application_id)
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: None,
                },
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    #[inline]
    pub async fn respond_embed(
        &self,
        module: &ModuleContext,
        embed: Embed,
        components: Option<Vec<Component>>,
    ) -> Result<(), HikariError> {
        let ic = module.discord.interaction(self.interaction.application_id);

        if self.responded.load(std::sync::atomic::Ordering::SeqCst) {
            let components_slice: Option<&[Component]> = components.as_deref();
            ic.update_response(&self.interaction.token)
                .embeds(Some(&[embed]))
                .components(components_slice)
                .await?;
        } else {
            self.responded
                .store(true, std::sync::atomic::Ordering::SeqCst);
            let mut builder = InteractionResponseDataBuilder::new().embeds([embed]);
            if let Some(comps) = components {
                builder = builder.components(comps);
            }
            ic.create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(builder.build()),
                },
            )
            .await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn respond_ephemeral(
        &self,
        module: &ModuleContext,
        content: &str,
    ) -> Result<(), HikariError> {
        let ic = module.discord.interaction(self.interaction.application_id);

        if self.responded.load(std::sync::atomic::Ordering::SeqCst) {
            ic.update_response(&self.interaction.token)
                .content(Some(content))
                .await?;
        } else {
            self.responded
                .store(true, std::sync::atomic::Ordering::SeqCst);
            ic.create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .content(content)
                            .flags(twilight_model::channel::message::MessageFlags::EPHEMERAL)
                            .build(),
                    ),
                },
            )
            .await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn defer_update(&self, module: &ModuleContext) -> Result<(), HikariError> {
        if self
            .responded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(());
        }

        module
            .discord
            .interaction(self.interaction.application_id)
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredUpdateMessage,
                    data: None,
                },
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}
