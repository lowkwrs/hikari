use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;

pub trait Subscriber: Send + Sync + 'static {
    fn subscribe(
        &self,
    ) -> impl std::future::Future<
        Output = Result<tokio::sync::broadcast::Receiver<HikariEvent>, HikariError>,
    > + Send;
}
