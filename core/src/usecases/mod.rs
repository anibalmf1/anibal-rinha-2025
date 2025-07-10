mod process_payment;
mod get_summary;

use process_payment::{ProcessPayment};
use crate::queue::{Producer};
use crate::outbound::PaymentProcessor;
use crate::store::PaymentStore;
use crate::usecases::get_summary::GetSummary;

#[derive(Clone, Debug)]
pub struct UseCases {
    pub process_payment: ProcessPayment,
    pub get_summary: GetSummary,
}

impl UseCases {
    pub async fn new(
        producer: Producer,
        payment_processor: PaymentProcessor,
        payment_store: PaymentStore,
    ) -> Self {
        Self{
            process_payment: ProcessPayment::new(producer, payment_processor, payment_store.clone()).await,
            get_summary: GetSummary::new(payment_store).await,
        }
    }
}
