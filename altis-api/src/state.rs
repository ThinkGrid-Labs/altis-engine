use std::sync::Arc;
use altis_infra::{DbClient, RedisClient, EventProducer};
use tokio::sync::broadcast;
use altis_domain::events::SeatHeldEvent;
use altis_domain::repository::FlightRepository;

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
    pub flight_repo: Arc<dyn FlightRepository>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub auth: AuthConfig,
    pub business_rules: altis_infra::config::BusinessRules,
}
