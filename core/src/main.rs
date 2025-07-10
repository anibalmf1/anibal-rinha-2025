mod config;
mod message;
mod routes;
mod usecases;
mod models;
mod outbound;
mod consumers;

use actix_web::{web, App, HttpServer};
use std::io::{Result};
use actix_web::middleware::Logger;
use tracing::info;
use tracing_subscriber::{fmt};

use config::{Settings};
use crate::message::{Producer,Consumer};
use crate::outbound::PaymentProcessor;
use crate::usecases::UseCases;

#[actix_web::main]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting anibalmf1-rust server");

    let settings = Settings::new();
    let producer= Producer::new(settings.clone()).await;
    let consumer= Consumer::new(settings.clone()).await;
    let payment_processor= PaymentProcessor::new(settings.payment_processor_url.clone()).await;
    let usecases = UseCases::new(producer, payment_processor, settings.clone()).await;
    let payment_consumer = consumers::PaymentConsumer::new(usecases.clone()).await;

    consumer.add_consumer(settings.payment_topic, payment_consumer).await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(usecases.clone()))
            .service(routes::process_payment)
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
