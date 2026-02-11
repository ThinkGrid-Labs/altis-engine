use std::sync::Arc;
use altis_store::{DbClient, RedisClient, EventProducer};
use tokio::sync::broadcast;
use altis_core::repository::FlightRepository;
use altis_shared::models::events::SeatHeldEvent;

#[derive(Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbClient>,
    pub redis: Arc<RedisClient>,
    pub kafka: Arc<EventProducer>,
    pub flight_repo: Arc<dyn FlightRepository + Send + Sync>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub auth: AuthConfig,
    pub business_rules: altis_store::app_config::BusinessRules,
}
