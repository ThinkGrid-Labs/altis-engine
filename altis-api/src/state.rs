use std::sync::Arc;
use altis_infra::{DbClient, RedisClient, EventProducer};
use tokio::sync::broadcast;
use altis_domain::events::SeatHeldEvent;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbClient>,
    pub redis: Arc<RedisClient>,
    pub kafka: Arc<EventProducer>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub business_rules: altis_infra::config::BusinessRules,
}
