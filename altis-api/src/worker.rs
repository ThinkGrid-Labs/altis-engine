use std::sync::Arc;
use tokio::time::{sleep, Duration};
use altis_infra::{EventProducer, RedisClient, DbClient}; 
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::message::Message;
use tracing::{info, error};
use uuid::Uuid;

pub async fn start_availability_worker(
    brokers: String,
    group_id: String,
    db: Arc<DbClient>,
    redis: Arc<RedisClient>
) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", &group_id)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&["booking.confirmed"]).expect("Can't subscribe");

    info!("Availability Worker started, listening to bookings...");

    loop {
        match consumer.recv().await {
            Err(e) => error!("Kafka error: {}", e),
            Ok(m) => {
                if let Some(payload) = m.payload_view::<str>() {
                    match payload {
                        Ok(booking_id_str) => {
                            info!("Processing booking update: {}", booking_id_str);
                            if let Ok(booking_id) = Uuid::parse_str(booking_id_str) {
                                // Update availability for this booking's flights
                                if let Err(e) = update_availability(&db, &redis, booking_id).await {
                                    error!("Failed to update availability: {}", e);
                                }
                            }
                        },
                        Err(e) => error!("Error reading payload: {}", e),
                    }
                }
            }
        }
    }
}

async fn update_availability(db: &DbClient, redis: &RedisClient, booking_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get flights associated with this booking
    // Enterprise Pattern: The event payload SHOULD contain flight_ids to avoid DB lookup.
    // Ideally: "booking.confirmed" -> { booking_id, flight_ids: [...] }
    // But our current producer sends just "booking_id".
    // We will stick to DB lookup for now (Reliability > Speed here).

    let flights = sqlx::query!(
        r#"
        SELECT DISTINCT flight_id 
        FROM passenger_seats 
        JOIN passengers ON passengers.id = passenger_seats.passenger_id
        WHERE passengers.booking_id = $1
        "#,
        booking_id
    )
    .fetch_all(&db.pool)
    .await?;

    for f in flights {
        let flight_id = f.flight_id;
        let key = format!("flight:{}:availability", flight_id);
        
        // 2. Atomic Decrement (Optimistic)
        // If the key exists (hot cache), this is O(1).
        // 2. Atomic Decrement (Optimistic)
        match redis.decr_flight_availability(&flight_id.to_string()).await {
             Ok(Some(new_val)) => {
                 info!("Decremented availability for flight {}: {}", flight_id, new_val);
             },
             Ok(None) => {
                 info!("Cache miss for flight {}, skipping decrement (will be seeded on next Search)", flight_id);
             },
             Err(e) => {
                 error!("Failed to decrement flight availability: {}", e);
             }
        }

    }
    
    Ok(())
}
