use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ResourcePool {
    semaphore: Arc<Semaphore>,
}

impl ResourcePool {
    #[inline]
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    #[inline(always)]
    pub async fn acquire(&self) -> Option<tokio::sync::OwnedSemaphorePermit> {
        self.semaphore.clone().acquire_owned().await.ok()
    }

    #[inline]
    pub fn try_acquire(&self) -> Option<tokio::sync::OwnedSemaphorePermit> {
        self.semaphore.clone().try_acquire_owned().ok()
    }

    #[inline]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

impl Clone for ResourcePool {
    fn clone(&self) -> Self {
        Self {
            semaphore: Arc::clone(&self.semaphore),
        }
    }
}
