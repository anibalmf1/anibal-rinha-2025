use async_trait::async_trait;
use tracing::info;
use crate::message::ConsumerHandler;
use crate::models::Payment;
use crate::usecases::UseCases;

pub struct PaymentConsumer{
    usecases: UseCases
}

impl PaymentConsumer {
    pub async fn new(usecases: UseCases) -> Self {
        Self{
            usecases,
        }
    }
}

#[async_trait]
impl ConsumerHandler for PaymentConsumer {
    async fn consume(&self, message: String) {
        info!("Consuming message:{}", message);
        
        let payment = serde_json::from_str::<Payment>(message.as_str()).unwrap();
        self.usecases.clone()
            .process_payment
            .execute(payment)
            .await
            .unwrap()
    }
}
