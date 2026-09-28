use crate::core::handler::ModuleCommand;
use hikari_utils::{error::HikariError, module::ModuleContext};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::{
        Message,
        message::{Embed, component::Component},
    },
    id::{Id, marker::GuildMarker},
};

pub struct PrefixContext {
    pub message: Message,
    pub args: Vec<String>,
    pub guild_id: Id<GuildMarker>,
    pub http: Arc<HttpClient>,
}

impl PrefixContext {
    pub async fn reply(&self, content: &str) -> Result<(), HikariError> {
        self.http
            .create_message(self.message.channel_id)
            .content(content)
            .await
            .map(|_| ())
            .map_err(|e| HikariError::Internal(e.to_string()))
    }

    pub async fn reply_with_ui(
        &self,
        embed: Embed,
        components: Vec<Component>,
    ) -> Result<(), HikariError> {
        self.http
            .create_message(self.message.channel_id)
            .embeds(&[embed])
            .components(&components)
            .await
            .map(|_| ())
            .map_err(|e| HikariError::Internal(e.to_string()))
    }
}

pub struct PrefixRouter {
    prefix: String,
}

impl PrefixRouter {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    pub fn parse_prefix<'a>(&self, content: &'a str) -> Option<(&'a str, &'a str)> {
        let content = content.trim();
        let bot_id = hikari_utils::ids::get_bot_id();

        let after_trigger = if content.starts_with(self.prefix.as_str()) {
            &content[self.prefix.len()..]
        } else if bot_id != 0 {
            let m1 = format!("<@{}> ", bot_id);
            let m2 = format!("<@!{}> ", bot_id);
            if content.starts_with(&m1) {
                &content[m1.len()..]
            } else if content.starts_with(&m2) {
                &content[m2.len()..]
            } else {
                return None;
            }
        } else {
            return None;
        };

        let after_trigger = after_trigger.trim_start();
        if after_trigger.is_empty() {
            return None;
        }

        let mut parts = after_trigger.splitn(2, char::is_whitespace);
        let name = parts.next()?;
        let rest = parts.next().unwrap_or("").trim();
        Some((name, rest))
    }

    pub async fn handle_message(
        &self,
        msg: &Message,
        ctx: &ModuleContext,
        prefix_lookup: &HashMap<&'static str, Arc<dyn ModuleCommand>>,
        builtin_names: &HashSet<&'static str>,
    ) -> Result<(), HikariError> {
        if msg.author.bot || msg.webhook_id.is_some() {
            return Ok(());
        }

        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let repo = hikari_premium::engine();
        let noprefix_enabled = repo.is_noprefix_enabled(guild_id.get() as i64).await;

        let (name, rest, is_noprefix) = if let Some(parsed) = self.parse_prefix(&msg.content) {
            (parsed.0, parsed.1, false)
        } else if noprefix_enabled {
            let content = msg.content.trim();
            let mut parts = content.splitn(2, char::is_whitespace);
            let first = parts.next().unwrap_or("");
            let rest = parts.next().unwrap_or("").trim();
            let first_lower = first.to_lowercase();

            if first_lower.is_empty()
                || (!prefix_lookup.contains_key(first_lower.as_str())
                    && !builtin_names.contains(first_lower.as_str()))
            {
                return Ok(());
            }
            (first, rest, true)
        } else {
            return Ok(());
        };

        if is_noprefix {
            let user_id = msg.author.id.get() as i64;
            let guild_id_i64 = guild_id.get() as i64;

            if !repo.has_noprefix_access(guild_id_i64, user_id, None).await {
                return Ok(());
            }

            let mut redis_conn = ctx
                .redis
                .get()
                .await
                .map_err(|e| HikariError::Internal(e.to_string()))?;
            let key = format!("noprefix_cd:{}:{}", guild_id_i64, user_id);
            let count = hikari_cache::models::limiter::Limiter::check_and_increment(
                &mut redis_conn,
                &key,
                2,
            )
            .await?;

            if count > 1 {
                return Ok(()); // Rate limited
            }
        }

        let args: Vec<String> = rest.split_whitespace().map(str::to_string).collect();

        let prefix_ctx = PrefixContext {
            message: msg.clone(),
            args,
            guild_id,
            http: ctx.discord.clone(),
        };

        let command_name = name.to_lowercase();
        if let Some(cmd) = prefix_lookup.get(command_name.as_str()) {
            cmd.handle_prefix(&prefix_ctx, ctx).await?;
        } else if builtin_names.contains(command_name.as_str()) {
            Self::handle_builtin(&prefix_ctx, &command_name).await?;
        }
        Ok(())
    }

    async fn handle_builtin(ctx: &PrefixContext, name: &str) -> Result<(), HikariError> {
        if name == "ping" {
            ctx.reply("🏓 Pong!").await?
        }
        Ok(())
    }
}
