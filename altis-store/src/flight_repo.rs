use altis_core::search::{FlightOption, SearchLeg};
use altis_core::repository::FlightRepository;
use async_trait::async_trait;
use std::error::Error;

pub struct PostgresFlightRepository {
    pub pool: sqlx::PgPool,
    pub redis: crate::RedisClient,
}

#[async_trait]
impl FlightRepository for PostgresFlightRepository {
    async fn search_flights(
        &self,
        leg: &SearchLeg,
        min_seats: u32,
    ) -> Result<Vec<FlightOption>, Box<dyn Error + Send + Sync>> {
        // 1. Get Flights Candidates (No COUNT yet)
        let flights = sqlx::query!(
            r#"
            SELECT 
                f.id, f.flight_number, f.scheduled_departure as "departure_time!", f.scheduled_arrival as "arrival_time!", 
                r.origin_iata as "origin_airport_code!", r.destination_iata as "destination_airport_code!",
                ac.name as "aircraft_model!",
                ac.capacity,
                f.base_price_amount, f.base_price_currency
            FROM flights f
            JOIN routes r ON f.route_id = r.id
            JOIN aircraft_configs ac ON f.aircraft_config_id = ac.id
            WHERE 
                r.origin_iata = $1 
                AND r.destination_iata = $2
                AND DATE(f.scheduled_departure) = $3
            "#,
            leg.origin_airport_code,
            leg.destination_airport_code,
            leg.date
        )
        .fetch_all(&self.pool)
        .await?;

        let mut options = Vec::new();
        
        for row in flights {
            let flight_id = row.id;
            let capacity = row.capacity;
            
            // 2. Access Redis for Availability
            let cached_availability: Option<i32> = self.redis.get_flight_availability(&flight_id.to_string()).await.ok().flatten();
            
            let remaining = match cached_availability {
                Some(count) => count as i64,
                None => {
                    // Fallback to SQL Count
                     let booked_rec = sqlx::query!(
                        r#"
                        SELECT COUNT(*) as "count!"
                        FROM passenger_seats 
                        WHERE flight_id = $1
                        "#,
                        flight_id
                    )
                    .fetch_one(&self.pool)
                    .await?;
                    
                    let count = (capacity as i64) - (booked_rec.count);
                    
                    // Populate Cache
                    let _ = self.redis.set_flight_availability(&flight_id.to_string(), count as i32).await;
                    
                    count
                }
            };
            
            if remaining >= (min_seats as i64) {
                options.push(FlightOption {
                    flight_id: row.id,
                    flight_number: row.flight_number,
                    departure_time: row.departure_time,
                    arrival_time: row.arrival_time,
                    origin: row.origin_airport_code,
                    destination: row.destination_airport_code,
                    aircraft_model: row.aircraft_model,
                    remaining_seats: remaining as i32,
                    price_amount: row.base_price_amount,
                    price_currency: row.base_price_currency,
                });
            }
        }

        Ok(options)
    }
}
