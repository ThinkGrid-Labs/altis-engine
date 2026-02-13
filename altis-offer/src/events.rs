use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::config::ClientConfig;
use std::time::Duration;
use altis_shared::models::events::{OfferGeneratedEvent, OfferAcceptedEvent};
use std::sync::Arc;

pub struct OfferTelemetry {
    producer: Arc<FutureProducer>,
    topic: String,
}

impl OfferTelemetry {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");
            
        Self {
            producer: Arc::new(producer),
            topic: topic.to_string(),
        }
    }

    pub async fn log_offer_generated(&self, event: OfferGeneratedEvent) -> Result<(), String> {
        self.publish("offer_generated", &event).await
    }

    pub async fn log_offer_accepted(&self, event: OfferAcceptedEvent) -> Result<(), String> {
        self.publish("offer_accepted", &event).await
    }

    pub async fn log_order_paid(&self, event: altis_shared::models::events::OrderPaidEvent) -> Result<(), String> {
        self.publish("order_paid", &event).await
    }

    pub async fn log_settlement(&self, event: altis_shared::models::events::SettlementEvent) -> Result<(), String> {
        self.publish("settlement", &event).await
    }

    async fn publish<T: serde::Serialize>(&self, event_type: &str, payload: &T) -> Result<(), String> {
        let json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        
        let record = FutureRecord::to(&self.topic)
            .payload(&json)
            .key(event_type);
            
        self.producer
            .send(record, Duration::from_secs(0))
            .await
            .map(|_| ())
            .map_err(|(e, _): (rdkafka::error::KafkaError, rdkafka::message::OwnedMessage)| e.to_string())
    }
}
