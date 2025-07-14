extern crate core;

mod config;
mod queue;
mod routes;
mod usecases;
mod models;
mod outbound;
mod consumers;
mod store;
mod serializers;

use actix_web::{web, App, HttpServer};
use std::io::{Result};
use actix_web::middleware::Logger;
use tracing::{info};
use tracing_subscriber::{fmt};

use config::{Settings};
use crate::queue::{Producer, Consumer, DLQConsumer};
use crate::outbound::PaymentProcessor;
use crate::usecases::UseCases;

#[actix_web::main]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting anibalmf1-rust server");

    let settings = Settings::new();

    let producer = Producer::new(settings.clone()).await;
    let consumer = Consumer::new(settings.clone()).await;
    let payment_processor = PaymentProcessor::new(settings.payment_processor_url.clone()).await;
    let payment_store = store::PaymentStore::new(settings.clone()).await;
    let usecases = UseCases::new(producer, payment_processor, payment_store).await;
    let payment_consumer = consumers::PaymentConsumer::new(usecases.clone()).await;
    let dlq_consumer = DLQConsumer::new(settings.clone()).await;

    // Start consuming messages from the queue
    consumer.start_consuming(payment_consumer.clone()).await;
    dlq_consumer.start_consuming(payment_consumer).await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(usecases.clone()))
            .service(routes::process_payment)
            .service(routes::get_summary)
    })
        .bind((settings.server_url.clone(), settings.server_port))?
        .run()
        .await
}

fn init_tracing() {
    fmt()
        .with_line_number(true)
        .init();
}
