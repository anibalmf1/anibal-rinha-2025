
use crate::message::{Producer};
use crate::models::{Payment};

struct ProcessPayment {
    producer: Producer
}

impl ProcessPayment {
    pub async fn new(producer: Producer) -> Self {
        Self {
            producer
        }
    }

    pub async fn execute(&self, payment: Payment) {
        
    }
}