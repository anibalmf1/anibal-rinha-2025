use crate::config::settings::Settings;

pub struct Producer {
    nats: nats::Connection
}

impl Producer {
    pub async fn new(settings: Settings) -> Self {
        let nats = nats::connect(settings.nats_url).await;

        Self {
            nats
        }
    }

    pub async fn publish(&self, topic: &str, message: &str) {
        self.publish(topic, message).await;
    }
}