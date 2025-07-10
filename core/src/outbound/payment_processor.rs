use reqwest::Client;
use crate::models::Payment;

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
        let payload = serde_json::to_string(&payment).unwrap();

        match self.client.post(&self.processor_url)
            .json(&payload)
            .send()
            .await {
            Ok(res) => {
                if res.status().is_success() {
                    Ok(())
                } else {
                    Err(res.text().await.unwrap())
                }
            },
            Err(e) => Err(e.to_string())
        }
    }
}