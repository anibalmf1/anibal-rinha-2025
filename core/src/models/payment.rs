use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
}
