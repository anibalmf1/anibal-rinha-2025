use serde::{Deserialize, Serialize};
use config::{Case, Config};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server_url: String,
    pub server_port: u16,
    pub redis_url: String,
    pub payment_processor_url: String,
    pub payment_topic: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
}

impl Settings {
    pub fn new() -> Self {
        let cfg = Config::builder()
            .add_source(config::Environment::with_prefix("APP")
                .convert_case(Case::Snake)
                .try_parsing(true)
            )
            .build().unwrap();

        cfg.try_deserialize().unwrap()
    }
}
