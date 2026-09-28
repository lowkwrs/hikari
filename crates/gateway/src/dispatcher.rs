use hikari_events::publisher::Publisher;
use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;
use std::sync::Arc;

pub struct EventDispatcher<P: Publisher> {
    publisher: Arc<P>,
}

impl<P: Publisher> EventDispatcher<P> {
    pub fn new(publisher: Arc<P>) -> Self {
        Self { publisher }
    }

    pub async fn dispatch(&self, event: HikariEvent) -> Result<(), HikariError> {
        self.publisher.publish(event).await
    }
}
