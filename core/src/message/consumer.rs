use std::ops::Deref;
use crate::config::settings::Settings;

pub struct Consumer {
    nats: nats::Connection
}

impl Consumer {
    pub async fn new(settings: Settings) -> Self {
        let nats = nats::connect(settings.nats_url).await;

        Self {
            nats
        }
    }

    pub async fn add_consumer(&self, topic: &str, consumer: fn(&str)) {
        let sub = self.nats.subscribe(topic).unwrap();

        let _ = tokio::task::spawn_blocking(|| {
            for msg in sub.messages() {
                let str_message = String::from_utf8_lossy(&msg.data);
                consumer(str_message.deref());
            }
        });
    }
}