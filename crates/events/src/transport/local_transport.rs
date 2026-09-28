use crate::publisher::Publisher;
use crate::subscriber::Subscriber;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use tokio::sync::broadcast;

pub struct LocalTransport {
    sender: broadcast::Sender<HikariEvent>,
}

impl LocalTransport {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
}

impl Publisher for LocalTransport {
    async fn publish(&self, event: HikariEvent) -> Result<(), HikariError> {
        if self.sender.receiver_count() > 0 {
            self.sender.send(event).map_err(|e| {
                HikariError::Internal(format!("Failed to publish local event: {}", e))
            })?;
        }
        Ok(())
    }
}

impl Subscriber for LocalTransport {
    async fn subscribe(&self) -> Result<broadcast::Receiver<HikariEvent>, HikariError> {
        Ok(self.sender.subscribe())
    }
}
