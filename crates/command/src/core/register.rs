use crate::antinuke::AntinukeHandler;
use crate::automod::AutomodHandler;
use crate::core::handler::ModuleCommand;
use crate::premium::PremiumHandler;

use hikari_utils::error::HikariError;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::guild::Permissions;

pub async fn register_global_commands(http: Arc<HttpClient>) -> Result<(), HikariError> {
    let app_id = http.current_user_application().await?.model().await?.id;
    let ic = http.interaction(app_id);

    let antinuke_handler = AntinukeHandler;
    let mut antinuke = antinuke_handler.command();
    antinuke.default_member_permissions = Some(Permissions::MANAGE_GUILD);

    let automod_handler = AutomodHandler;
    let mut automod = automod_handler.command();
    automod.default_member_permissions = Some(Permissions::MANAGE_GUILD);

    let premium_handler = PremiumHandler::new();
    let premium = premium_handler.command();

    let ping_handler = crate::misc::PingHandler;
    let ping = ping_handler.command();

    let help_handler = crate::misc::HelpHandler;
    let help = help_handler.command();

    ic.set_global_commands(&[antinuke, automod, premium, ping, help])
        .await?;

    tracing::info!("Global application commands registered successfully.");
    Ok(())
}
