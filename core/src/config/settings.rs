use serde::{Deserialize, Serialize};
use config::{Config,ConfigError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub server_url: String,
    pub server_port: u16,
    pub nats_url: String,
    pub payment_processor_default: String,
    pub payment_processor_fallback: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let cfg = Config::builder()
            .add_source(config::Environment::with_prefix("APP")
                .separator("_")
                .try_parsing(true)
            )
            .build()?;

        cfg.try_deserialize()
    }
}