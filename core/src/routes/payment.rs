use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::models::Payment;
use crate::usecases::UseCases;

#[derive(Deserialize)]
pub struct SummaryParams {
    from: Option<String>,
    to: Option<String>,
}

#[post("/payments")]
pub async fn process_payment(
    usecases: web::Data<UseCases>, 
    payload: web::Json<Payment>
) -> impl Responder {
    match usecases.process_payment.clone().execute(payload.0, true).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Payment processing failed but was queued for retry: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

#[get("/payments-summary")]
pub async fn get_summary(usecases: web::Data<UseCases>, query: web::Query<SummaryParams>) -> impl Responder {
    let summary = usecases.get_summary.clone().execute(query.from.clone(), query.to.clone()).await;

    HttpResponse::Ok().json(summary)
}
