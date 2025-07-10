mod process_payment;

use process_payment::{ProcessPayment};
use crate::config::Settings;
use crate::message::Producer;
use crate::outbound::PaymentProcessor;

#[derive(Clone, Debug)]
pub struct UseCases {
    pub process_payment: ProcessPayment
}

impl UseCases {
    pub async fn new(
        producer: Producer,
        payment_processor: PaymentProcessor,
        settings: Settings,
    ) -> Self {
        Self{
            process_payment: ProcessPayment::new(producer, payment_processor, settings).await,
        }
    }
}