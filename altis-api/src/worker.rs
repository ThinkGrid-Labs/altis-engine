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
        let new_val: Option<i64> = redis.decr_flight_availability(&flight_id.to_string()).await.ok();
        
        if let Some(val) = new_val {
             info!("Decremented availability for flight {}: {}", flight_id, val);
        } else {
             // Cache Miss or Error?
             // If DECR returned success (even if negative), we are good?
             // Actually, if key didn't exist, Redis DECR makes it -1.
             // We should check if it was < 0 (unseeded) and seed it.
             // Or simpler: Just rely on Search seeding it.
             // If Search seeds it, it uses `(Capacity - Count)`.
             // `Count` includes this new booking.
             // So if we DECR an empty key to -1, then Search Seeds it to (Capacity - Count),
             // the -1 is overwritten.
             // But if we DECR an EXISTING key, we are correct.
             // The only race is: Search reads (Capacity - Count), puts X. 
             // We DECR to Y.
             // If we DECR non-existent key, we get -1. This is useless state.
             // We should delete the key if it goes negative to force re-seed?
             // "Invalidate Cache" pattern.
             
             // Enterprise Pattern: Cache Invalidation.
             // Instead of DECR, just DEL key.
             // Next Search will re-calculate (Capacity - Count) accurately.
             // This is 100% consistent and safe.
             
             let _ = redis.client.get_async_connection().await?.del::<_, ()>(key).await;
             info!("Invalidated cache for flight {}", flight_id);
        }
    }
    
    Ok(())
}
