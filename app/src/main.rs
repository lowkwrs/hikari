use hikari_command::core::router::CommandRouter;
use hikari_database::pool::Database;
use hikari_events::subscriber::Subscriber;
use hikari_events::transport::local_transport::LocalTransport;
use hikari_features_registry::ModuleRegistry;
use hikari_gateway::dispatcher::EventDispatcher;
use hikari_gateway::event_loop::EventLoop;
use hikari_gateway::shard_manager::ShardManager;
use hikari_utils::config::AppConfig;
use hikari_utils::error::HikariError;
use hikari_utils::module::ModuleContext;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::oneshot;
use tracing::{error, info};
use twilight_http::Client as DiscordClient;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[inline]
fn sanitize_postgres_url(url: &str) -> String {
    if let Some(pos) = url.find('@') {
        format!("postgresql://*****@{}", &url[pos + 1..])
    } else {
        url.to_string()
    }
}

#[inline]
fn sanitize_redis_url(url: &str) -> String {
    url.find("://")
        .and_then(|pos| {
            let rest = &url[pos + 3..];
            rest.find('@')
                .map(|at_pos| format!("{}*****@{}", &url[..pos + 3], &rest[at_pos + 1..]))
        })
        .unwrap_or_else(|| url.to_string())
}

#[tokio::main]
async fn main() -> Result<(), HikariError> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = AppConfig::from_env()?;

    info!(
        "[POSTGRES] Connecting to database: {}",
        sanitize_postgres_url(&config.database_url)
    );
    let db_wrapper = Database::connect(&config.database_url).await?;
    let db = db_wrapper.pool.clone();

    info!("[POSTGRES] Running database migrations...");
    db_wrapper.run_migrations().await.map_err(|e| {
        error!("[POSTGRES] Failed to run migrations: {}", e);
        HikariError::Database(e)
    })?;
    info!("[POSTGRES] Migrations complete.");

    info!(
        "[REDIS] Connecting to server: {}",
        sanitize_redis_url(&config.redis_url)
    );
    let redis_pool = hikari_cache::RedisPool::new(&config.redis_url)?;
    let redis = redis_pool.pool.clone();

    let discord = Arc::new(DiscordClient::new(config.discord_token.clone()));

    hikari_utils::executor::ConnectionWarmer::warm_connections(discord.clone(), 5).await;

    let bot_user: twilight_model::user::CurrentUser = discord.current_user().await?.model().await?;
    let bot_name = bot_user.name.clone();
    hikari_utils::ids::set_bot_id(bot_user.id.get());

    info!("[DISCORD] Registering global slash commands...");
    hikari_command::register::register_global_commands(discord.clone()).await?;
    info!("[DISCORD] Global slash commands registered successfully");

    info!("[TRANSPORT] Initializing local messaging transport (Buffer: 1024)...");
    let local_transport = Arc::new(LocalTransport::new(1024));

    let module_ctx = Arc::new(ModuleContext {
        db,
        redis,
        discord,
        embed_color: hikari_embeds::color::PRIMARY,
    });

    let registry = Arc::new(ModuleRegistry::new(
        module_ctx.discord.clone(),
        db_wrapper.clone(),
        redis_pool.clone(),
    ));

    let _premium_module = hikari_premium::PremiumModule::new(db_wrapper.clone());

    let mut rx = local_transport.subscribe().await?;
    let registry_ctx = module_ctx.clone();
    let registry_pool = hikari_utils::pool::ResourcePool::new(128);

    tokio::spawn(async move {
        info!("[MODULES] Module registry listening for events...");
        while let Ok(event) = rx.recv().await {
            let registry = Arc::clone(&registry);
            let registry_ctx = Arc::clone(&registry_ctx);
            if let Some(permit) = registry_pool.try_acquire() {
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = registry.handle_event(&event, &registry_ctx).await;
                });
            }
        }
    });

    let mut command_router = CommandRouter::new(config.prefix.clone());
    command_router.register(Arc::new(hikari_command::antinuke::AntinukeHandler));
    command_router.register(Arc::new(hikari_command::automod::AutomodHandler));
    command_router.register(Arc::new(hikari_command::premium::PremiumHandler::new()));
    command_router.register(Arc::new(hikari_command::misc::PingHandler));
    command_router.register(Arc::new(hikari_command::misc::HelpHandler));
    let command_router = Arc::new(command_router);
    let mut cmd_rx = local_transport.subscribe().await?;
    let cmd_ctx = module_ctx.clone();
    let cmd_pool = hikari_utils::pool::ResourcePool::new(128);

    tokio::spawn(async move {
        info!("[COMMANDS] Router listening for events...");
        while let Ok(event) = cmd_rx.recv().await {
            let command_router = Arc::clone(&command_router);
            let cmd_ctx = Arc::clone(&cmd_ctx);
            if let Some(permit) = cmd_pool.try_acquire() {
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = command_router.handle_event(&event, &cmd_ctx).await;
                });
            }
        }
    });

    info!("[GATEWAY] Initializing Shard Manager with recommended sharding...");
    let shard_manager =
        ShardManager::new(config.discord_token.clone(), &module_ctx.discord).await?;
    let dispatcher = EventDispatcher::new(local_transport.clone());
    let event_loop = EventLoop::new(shard_manager, dispatcher);

    let (ready_tx, ready_rx) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = event_loop.run(Some(ready_tx)).await {
            error!("[GATEWAY] Gateway event loop crashed: {}", e);
        }
    });

    ready_rx.await.ok();
    info!("[SYSTEM] Connected to Discord as {}", bot_name);
    info!("[SYSTEM] Waiting for shutdown signal...");

    match signal::ctrl_c().await {
        Ok(()) => info!("[SYSTEM] Shutdown signal received. Exiting gracefully..."),
        Err(err) => error!("[SYSTEM] Unable to listen for shutdown signal: {}", err),
    }

    Ok(())
}
