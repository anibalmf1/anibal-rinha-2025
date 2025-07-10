use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use crate::config::Settings;
use crate::models::{Payment, PaymentMetric, PaymentProcessorName, PaymentSummary};
use uuid::Uuid;
use chrono::{DateTime, NaiveDateTime, Utc};
use tracing::error;

#[derive(Clone, Debug)]
pub struct PaymentStore {
    db_pool: Pool
}

impl PaymentStore {
    pub async fn new(settings: Settings) -> Self {
        let mut db_config = Config::new();
        db_config.host = Some(settings.db_host);
        db_config.port = Some(settings.db_port);
        db_config.dbname = Some(settings.db_name);
        db_config.user = Some(settings.db_user);
        db_config.password = Some(settings.db_password);

        let pool = db_config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

        _ = pool.get().await.unwrap();

        Self {
            db_pool: pool
        }
    }

    pub async fn create_payment(&self, payment: Payment, payment_processor_name: String) {
        let client = self.db_pool.get().await.unwrap();

        let payment_processor = match payment_processor_name.as_str() {
            "default" => PaymentProcessorName::Default.to_string(),
            "fallback" => PaymentProcessorName::Fallback.to_string(),
            _ => PaymentProcessorName::Default.to_string(),
        };

        let correlation_id = match Uuid::parse_str(&payment.correlation_id) {
            Ok(uuid) => uuid,
            Err(e) => {
                tracing::error!("Failed to parse UUID '{}': {}", payment.correlation_id, e);
                Uuid::new_v4()
            }
        };

        let requested_at = match DateTime::parse_from_rfc3339(&payment.requested_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                tracing::error!("Failed to parse timestamp '{}': {}", payment.requested_at, e);
                Utc::now()
            }
        };

        // Convert DateTime<Utc> to NaiveDateTime for PostgreSQL compatibility
        let naive_requested_at = requested_at.naive_utc();

        match client.execute(
            "INSERT INTO payments (correlation_id, payment_processor, amount, requested_at) VALUES ($1, $2, $3, $4)",
            &[&correlation_id, &payment_processor, &payment.amount, &naive_requested_at],
        ).await {
            Ok(_) => tracing::info!("Payment inserted successfully"),
            Err(e) => tracing::error!("Failed to insert payment: {}", e),
        };
    }

    fn build_metrics_query(&self, from: &Option<String>, to: &Option<String>) -> (String, Vec<chrono::NaiveDateTime>) {
        let mut query = String::from(
            "SELECT payment_processor, COUNT(1) as count, SUM(amount) as total_amount
             FROM payments"
        );

        let mut params = Vec::new();
        let mut param_index = 1;

        // Process from date if provided
        let mut from_parsed = None;
        if let Some(from_date) = from {
            let parsed = DateTime::parse_from_rfc3339(&from_date)
                .map(|dt| dt.naive_utc())
                .or_else(|_| NaiveDateTime::parse_from_str(&from_date, "%Y-%m-%dT%H:%M:%S"));

            match parsed {
                Ok(dt) => {
                    from_parsed = Some(dt);
                }
                Err(e) => {
                    error!("Failed to parse date `{}`: {}", from_date, e);
                }
            }
        }

        // Process to date if provided
        let mut to_parsed = None;
        if let Some(to_date) = to {
            let parsed = DateTime::parse_from_rfc3339(&to_date)
                .map(|dt| dt.naive_utc())
                .or_else(|_| NaiveDateTime::parse_from_str(&to_date, "%Y-%m-%dT%H:%M:%S"));

            match parsed {
                Ok(dt) => {
                    to_parsed = Some(dt);
                }
                Err(e) => {
                    error!("Failed to parse date `{}`: {}", to_date, e);
                }
            }
        }

        // Add WHERE clause only if we have at least one valid date
        if from_parsed.is_some() || to_parsed.is_some() {
            query.push_str(" WHERE");

            if let Some(parsed_date) = from_parsed {
                query.push_str(&format!(" requested_at >= ${}", param_index));
                params.push(parsed_date);
                param_index += 1;
            }

            if let Some(parsed_date) = to_parsed {
                if param_index > 1 {
                    query.push_str(" AND");
                }
                query.push_str(&format!(" requested_at <= ${}", param_index));
                params.push(parsed_date);
            }
        }

        query.push_str(" GROUP BY payment_processor");

        (query, params)
    }

    fn process_metrics_results(&self, rows: Vec<tokio_postgres::Row>) -> PaymentSummary {
        let mut default_metric: PaymentMetric = PaymentMetric { total_requests: 0, total_amount: Default::default() };
        let mut fallback_metric: PaymentMetric = PaymentMetric { total_requests: 0, total_amount: Default::default() };

        for row in rows {
            let payment_processor: String = row.get(0);

            if payment_processor == PaymentProcessorName::Default.to_string() {
                let requests: i64 = row.get(1);
                default_metric.total_requests = requests.try_into().unwrap();
                default_metric.total_amount = row.get(2);
            } else {
                let requests: i64 = row.get(1);
                fallback_metric.total_requests = requests.try_into().unwrap();
                fallback_metric.total_amount = row.get(2);
            }
        }

        PaymentSummary{ default: default_metric, fallback: fallback_metric }
    }

    pub async fn get_metrics(&self, from: Option<String>, to: Option<String>) -> PaymentSummary {
        let client = self.db_pool.get().await.unwrap();

        let (query, params) = self.build_metrics_query(&from, &to);

        let rows = client.query(&query, &params.iter().map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync)).collect::<Vec<_>>()).await.unwrap();

        self.process_metrics_results(rows)
    }
}
