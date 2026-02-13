use std::sync::Arc;
use altis_store::{RedisClient, EventProducer};
use crate::middleware::resiliency::CircuitBreaker;
use tokio::sync::{broadcast, Mutex};
use altis_shared::models::events::SeatHeldEvent;
use altis_core::repository::{OfferRepository, OrderRepository, ProductRepository};
use altis_offer::ai_ranker::OfferRanker;
use altis_offer::events::OfferTelemetry;

#[derive(Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub expiration: u64,
}

pub struct ResiliencyState {
    pub payment_cb: CircuitBreaker,
    pub ndc_cb: CircuitBreaker,
}

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisClient>,
    pub kafka: Arc<EventProducer>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub auth: AuthConfig,
    pub business_rules: altis_store::app_config::BusinessRules,
    pub offer_repo: Arc<dyn OfferRepository>,
    pub order_repo: Arc<dyn OrderRepository>,
    pub catalog_repo: Arc<dyn ProductRepository>,
    pub telemetry: Arc<OfferTelemetry>,
    pub ranker: Arc<Mutex<OfferRanker>>,
    pub payment_orchestrator: Arc<altis_order::orchestrator::PaymentOrchestrator>,
    pub one_id_resolver: Arc<dyn altis_core::identity::OneIdResolver>,
    pub resiliency: Arc<ResiliencyState>,
}
