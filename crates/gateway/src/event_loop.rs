use crate::dispatcher::EventDispatcher;
use crate::shard_manager::ShardManager;
use hikari_events::publisher::Publisher;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{error, info};
use twilight_gateway::StreamExt as _;

pub struct EventLoop<P: Publisher> {
    shard_manager: ShardManager,
    dispatcher: EventDispatcher<P>,
}

impl<P: Publisher> EventLoop<P> {
    pub fn new(shard_manager: ShardManager, dispatcher: EventDispatcher<P>) -> Self {
        Self {
            shard_manager,
            dispatcher,
        }
    }
}

impl<P: Publisher + 'static> EventLoop<P> {
    pub async fn run(self, ready_tx: Option<oneshot::Sender<()>>) -> Result<(), HikariError> {
        info!("Starting gateway event loop");

        let mut join_set = tokio::task::JoinSet::new();
        let dispatcher = Arc::new(self.dispatcher);
        let ready_tx = Arc::new(std::sync::Mutex::new(ready_tx));

        for mut shard in self.shard_manager.shards {
            let dispatcher = Arc::clone(&dispatcher);
            let ready_tx = Arc::clone(&ready_tx);
            join_set.spawn(async move {
                let shard_id = shard.id();
                info!("Starting event loop for shard {}", shard_id.number());

                while let Some(event_result) = shard
                    .next_event(twilight_gateway::EventTypeFlags::all())
                    .await
                {
                    match event_result {
                        Ok(event) => {
                            if matches!(event, twilight_model::gateway::event::Event::Ready(_))
                                && let Ok(mut guard) = ready_tx.lock()
                                && let Some(tx) = guard.take()
                            {
                                let _ = tx.send(());
                            }
                            let hikari_event = HikariEvent::from(event);
                            let _ = dispatcher.dispatch(hikari_event).await;
                        }
                        Err(_e) => {
                            error!("Shard {} error", shard_id.number());
                        }
                    }
                }
            });
        }

        while let Some(res) = join_set.join_next().await {
            if let Err(e) = res {
                error!("Shard task failed: {}", e);
            }
        }

        Ok(())
    }
}
