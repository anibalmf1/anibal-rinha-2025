use crate::config::Settings;
use bytes::Bytes;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Producer {
    nats: async_nats::Client
}

impl Producer {
    pub async fn new(settings: Settings) -> Self {
        let nats = async_nats::connect(settings.nats_url).await.unwrap();

        Self {
            nats
        }
    }

    pub async fn publish(&self, topic: String, message: String) -> Result<(), String>{
        let b_message = Bytes::from(message);

        info!("Publishing message to topic: {}", topic);

        self.nats.publish(topic, b_message).await.map_err(|e| e.to_string())
    }
}