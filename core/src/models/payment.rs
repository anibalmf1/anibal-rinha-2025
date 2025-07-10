use rust_decimal::Decimal;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

pub enum PaymentProcessorName {
    Default,
    Fallback,
}

impl Display for PaymentProcessorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PaymentProcessorName::Default => "default".to_string(),
            PaymentProcessorName::Fallback => "fallback".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    pub amount: Decimal,
    #[serde(rename = "requestedAt")]
    #[serde(default)]
    pub requested_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMetric {
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "totalAmount")]
    #[serde(serialize_with = "crate::serializers::decimal::serialize")]
    pub total_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentSummary {
    pub default: PaymentMetric,
    pub fallback: PaymentMetric,
}
