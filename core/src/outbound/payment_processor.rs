use reqwest::Client;
use tracing::error;
use crate::models::Payment;

#[derive(Clone, Debug)]
pub struct PaymentProcessor {
    client: Client,
    processor_url: String,
}

impl PaymentProcessor {
    pub async fn new(processor_url: String) -> Self {
        Self {
            client: Client::new(),
            processor_url,
        }
    }

    pub async fn process(self, payment: Payment) -> Result<(), String> {
        match self.client.post(&self.processor_url)
            .json(&payment)
            .send()
            .await {
            Ok(res) => {
                if res.status().is_success() {
                    Ok(())
                } else {
                    let error = res.text().await.unwrap();
                    error!("Error processing payment on payment_processor: {}", error);
                    Err(error)
                }
            },
            Err(e) => {
                error!("Error processing payment on payment_processor: {}", e);
                Err(e.to_string())
            }
        }
    }
}