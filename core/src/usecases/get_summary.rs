use crate::models::{PaymentSummary};
use crate::store::PaymentStore;

#[derive(Clone, Debug)]
pub struct GetSummary {
    payment_store: PaymentStore,
}

impl GetSummary {
    pub async fn new(
        payment_store: PaymentStore,
    ) -> Self {
        Self {
            payment_store
        }
    }

    pub async fn execute(self, from: Option<String>, to: Option<String>) -> PaymentSummary {
        self.payment_store.get_metrics(from, to).await
    }
}
