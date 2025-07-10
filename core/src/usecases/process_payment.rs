use tracing::error;
use crate::config::Settings;
use crate::message::{Producer};
use crate::models::{Payment};
use crate::outbound::PaymentProcessor;

#[derive(Clone, Debug)]
pub struct ProcessPayment {
    producer: Producer,
    payment_processor: PaymentProcessor,
    payment_topic: String,
}

impl ProcessPayment {
    pub async fn new(
        producer: Producer,
        payment_processor: PaymentProcessor,
        settings: Settings
    ) -> Self {
        Self {
            producer,
            payment_processor,
            payment_topic: settings.payment_topic
        }
    }

    pub async fn execute(self, payment: Payment) -> Result<(), String>{
        match self.payment_processor.process(payment.clone()).await {
            Ok(_) => Ok(()),
            Err(_) => {
                let payload = serde_json::to_string(&payment).unwrap();
                self.producer.publish(self.payment_topic.clone(), payload).await.map_err(|e| {
                    error!("failed to publish payment");
                    e.to_string()
                })
            }
        }
    }
}