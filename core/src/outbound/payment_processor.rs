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

    pub async fn process(self, payment: Payment) -> Result<String, String> {
        match self.client.post(&self.processor_url)
            .json(&payment)
            .send()
            .await {
            Ok(res) => {
                let status = res.status();
                // Extract the payment processor header
                let payment_processor = res.headers()
                    .get("x-payment-processor")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("default")
                    .to_string();

                if status.is_success() {
                    Ok(payment_processor)
                } else {
                    let error = res.text().await.unwrap();
                    error!("Error processing payment on payment_processor: {} - status: {}", error, status.as_str());
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
