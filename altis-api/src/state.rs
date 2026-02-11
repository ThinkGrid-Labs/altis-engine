use std::sync::Arc;
use altis_store::{RedisClient, EventProducer};
use tokio::sync::broadcast;
use altis_shared::models::events::SeatHeldEvent;

#[derive(Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisClient>,
    pub kafka: Arc<EventProducer>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub auth: AuthConfig,
    pub business_rules: altis_store::app_config::BusinessRules,
}
