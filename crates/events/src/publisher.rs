use hikari_utils::error::HikariError;
use hikari_utils::event::HikariEvent;

pub trait Publisher: Send + Sync {
    fn publish(
        &self,
        event: HikariEvent,
    ) -> impl std::future::Future<Output = Result<(), HikariError>> + Send;
}
