use async_trait::async_trait;
use hikari_utils::{error::HikariError, module::ModuleContext};
use twilight_model::application::command::{Command, CommandOption};
use twilight_model::application::interaction::application_command::CommandDataOption;

use crate::{interaction::InteractionContext, prefix::PrefixContext};

#[async_trait]
pub trait ModuleCommand: Send + Sync {
    fn name(&self) -> &'static str;

    fn command(&self) -> Command;

    /// Extra names this module handles as prefix/noprefix commands.
    /// Override to register aliases like "ban", "kick", "mute" etc.
    fn prefix_aliases(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn handle_slash(
        &self,
        ctx: InteractionContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError>;

    async fn handle_prefix(
        &self,
        ctx: &PrefixContext,
        module: &ModuleContext,
    ) -> Result<(), HikariError>;

    async fn handle_component(
        &self,
        ctx: &InteractionContext,
        custom_id: &str,
        module: &ModuleContext,
    ) -> Result<bool, HikariError>;
}

#[async_trait]
pub trait SubCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn build_option(&self) -> CommandOption;
    async fn execute(
        &self,
        interaction_ctx: &InteractionContext,
        ctx: &ModuleContext,
        options: &[CommandDataOption],
    ) -> Result<(), HikariError>;
}
