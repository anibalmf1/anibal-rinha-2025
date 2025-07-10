use async_trait::async_trait;
use futures::StreamExt;
use crate::config::Settings;

pub struct Consumer {
    nats: async_nats::Client
}

#[async_trait]
pub trait ConsumerHandler: Send + Sync + 'static {
    async fn consume(&self, message: String);
}

impl Consumer {
    pub async fn new(settings: Settings) -> Self {
        let nats = async_nats::connect(settings.nats_url).await.unwrap();

        Self {
            nats
        }
    }

    pub async fn add_consumer(&self, topic: String, consumer: impl ConsumerHandler) {
        let mut sub = self.nats.subscribe(topic).await.unwrap();

        _ = tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                consumer.consume(String::from_utf8(msg.payload.to_vec()).unwrap()).await;
            }
        })
    }
}