use async_trait::async_trait;
use tracing::{info, error};
use crate::queue::QueueConsumerHandler;
use crate::models::Payment;
use crate::usecases::UseCases;

pub struct PaymentConsumer{
    usecases: UseCases,
}

impl PaymentConsumer {
    pub async fn new(usecases: UseCases) -> Self {
        Self{
            usecases,
        }
    }
}

#[async_trait]
impl QueueConsumerHandler for PaymentConsumer {
    async fn consume(&self, message: String) -> Result<(), String> {
        info!("Consuming message: {}", message);

        let payment = match serde_json::from_str::<Payment>(message.as_str()) {
            Ok(p) => p,
            Err(e) => {
                let error_msg = format!("Failed to parse payment message: {}", e);
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        match self.usecases.clone().process_payment.execute(payment, false).await {
            Ok(_) => {
                info!("Payment processed successfully");
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to process payment: {}", e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }
}
