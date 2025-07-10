use async_trait::async_trait;

#[async_trait]
pub trait QueueConsumerHandler: Send + Sync + 'static {
    async fn consume(&self, message: String) -> Result<(), String>;
}

mod redis;

pub use redis::{Producer, Consumer};
