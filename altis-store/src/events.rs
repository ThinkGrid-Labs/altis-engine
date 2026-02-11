use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;
use tracing::{info, error};



#[derive(Clone)]
pub struct EventProducer {
    producer: FutureProducer,
}

impl EventProducer {
    pub fn new(brokers: &str) -> Result<Self, rdkafka::error::KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()?;

        Ok(Self { producer })
    }

    pub async fn publish(&self, topic: &str, key: &str, payload: &str) -> Result<(), rdkafka::error::KafkaError> {
        let record = FutureRecord::to(topic)
            .key(key)
            .payload(payload);

        match self.producer.send(record, Timeout::After(Duration::from_secs(0))).await {
            Ok(delivery) => {
                let partition = delivery.partition;
                let offset = delivery.offset;
                info!("Sent message to {}/{}: partition {} offset {}", topic, key, partition, offset);
                Ok(())
            }
            Err((e, _msg)) => {
                error!("Failed to send message to {}: {}", topic, e);
                Err(e)
            }
        }
    }
}
