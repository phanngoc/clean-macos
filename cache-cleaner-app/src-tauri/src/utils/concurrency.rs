use std::sync::Arc;
use tokio::sync::Semaphore;

pub const DEFAULT_CONCURRENCY: usize = 8;

pub fn create_semaphore(limit: usize) -> Arc<Semaphore> {
    Arc::new(Semaphore::new(limit))
}
