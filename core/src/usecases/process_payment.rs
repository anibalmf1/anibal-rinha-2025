use chrono::Utc;
use tracing::error;
use crate::queue::{Producer};
use crate::models::{Payment};
use crate::outbound::PaymentProcessor;
use crate::store::PaymentStore;

#[derive(Clone, Debug)]
pub struct ProcessPayment {
    producer: Producer,
    payment_processor: PaymentProcessor,
    payment_store: PaymentStore,
}

impl ProcessPayment {
    pub async fn new(
        producer: Producer,
        payment_processor: PaymentProcessor,
        payment_store: PaymentStore,
    ) -> Self {
        Self {
            producer,
            payment_processor,
            payment_store
        }
    }

    pub async fn execute(self, mut payment: Payment, publish_on_failure: bool) -> Result<(), String>{
        payment.requested_at = Utc::now().to_rfc3339().clone();

        match self.payment_processor.process(payment.clone()).await {
            Ok(payment_processor) => {
                self.payment_store.create_payment(payment, payment_processor).await;
                Ok(())
            },
            Err(e) => {
                if publish_on_failure {
                    let payload = serde_json::to_string(&payment).unwrap();
                    self.producer.publish(payload).await.map_err(|e| {
                        error!("failed to publish payment to queue");
                        e.to_string()
                    })
                } else {
                    // When coming from consumer, don't republish, just return the error
                    Err(format!("Payment processing failed: {}", e))
                }
            }
        }
    }
}
