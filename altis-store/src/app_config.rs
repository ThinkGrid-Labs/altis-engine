use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub auth: AuthConfig,
    pub business_rules: BusinessRules,
    pub ranking: RankingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RankingConfig {
    pub conversion_weight: f64,
    pub margin_weight: f64,
    pub ml_experiment_percentage: f64,
    pub ml_service_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BusinessRules {
    pub trip_hold_seconds: u64,
    pub seat_hold_seconds: u64,
    pub tax_rate: f64,
    pub booking_fee: f64,
    #[serde(default = "default_multiplier")]
    pub pricing_multiplier: f64, 
    #[serde(default)]
    pub pricing_adjustment: f64,
    pub sale_start: Option<String>, // ISO 8601
    pub sale_end: Option<String>,   // ISO 8601
}

fn default_multiplier() -> f64 { 1.0 }

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = config::Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(config::File::with_name("config/default"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(config::File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(config::File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1` would set the `debug` key
            .add_source(config::Environment::with_prefix("ALTIS").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
