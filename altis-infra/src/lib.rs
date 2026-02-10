pub mod config;
pub mod database;
pub mod redis_repo;
pub mod events;
pub mod booking_repo;

pub mod booking_repo;
pub mod flight_repo;

pub use database::DbClient;
pub use redis_repo::RedisClient;
pub use events::EventProducer;
pub use booking_repo::BookingRepository;
pub use flight_repo::PostgresFlightRepository;

pub fn hello() {
    println!("Hello from Altis Infra!");
    pub async fn fetch_business_rules(&self, defaults: crate::config::BusinessRules) -> Result<crate::config::BusinessRules, sqlx::Error> {
        // Struct to map SQL result
        struct RuleRow {
            rule_key: String,
            rule_value: serde_json::Value,
        }

        let rows = sqlx::query_as!(RuleRow, "SELECT rule_key, rule_value FROM business_rules")
            .fetch_all(&self.pool)
            .await?;

        let mut rules = defaults;

        for row in rows {
            let val = row.rule_value; // serde_json::Value
            
            // Expected format: {"value": <number/string>}
            if let Some(v) = val.get("value") {
                match row.rule_key.as_str() {
                    "pricing_multiplier" => if let Some(f) = v.as_f64() { rules.pricing_multiplier = f; },
                    "pricing_adjustment" => if let Some(f) = v.as_f64() { rules.pricing_adjustment = f; },
                    "tax_rate" => if let Some(f) = v.as_f64() { rules.tax_rate = f; },
                    "booking_fee" => if let Some(f) = v.as_f64() { rules.booking_fee = f; },
                    "trip_hold_seconds" => if let Some(u) = v.as_u64() { rules.trip_hold_seconds = u; },
                    "seat_hold_seconds" => if let Some(u) = v.as_u64() { rules.seat_hold_seconds = u; },
                    "sale_start" => if let Some(s) = v.as_str() { rules.sale_start = Some(s.to_string()); },
                    "sale_end" => if let Some(s) = v.as_str() { rules.sale_end = Some(s.to_string()); },
                    _ => {}
                }
            }
        }

        Ok(rules)
    }
}
