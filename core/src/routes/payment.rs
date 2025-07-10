use actix_web::{post, web, HttpResponse, Responder};
use crate::models::Payment;
use crate::usecases::UseCases;

#[post("/payments")]
pub async fn process_payment(usecases: web::Data<UseCases>, payload: web::Json<Payment>) -> impl Responder {
    match usecases.process_payment.clone().execute(payload.0).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}