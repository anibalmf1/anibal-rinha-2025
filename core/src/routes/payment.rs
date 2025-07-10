
use actix_web::{post};

#[post("/payment")]
pub fn payment(usecases: actix_web::web::Data<UseCases>) {
    
}