use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tracing::info;
use serde_json::Value;



#[derive(Clone)]
pub struct DbClient {
    pub pool: Pool<Postgres>,
}

impl DbClient {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(connection_string)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        info!("Running database migrations...");
        sqlx::migrate!("../migrations")
            .run(&self.pool)
            .await?;
        info!("Migrations completed successfully.");
        Ok(())
    }

    pub async fn fetch_business_rules(&self, defaults: crate::app_config::BusinessRules) -> Result<crate::app_config::BusinessRules, sqlx::Error> {
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
                    "pricing_multiplier" => {
                        if let Some(f) = Value::as_f64(v) {
                            rules.pricing_multiplier = f;
                        }
                    }
                    "pricing_adjustment" => {
                        if let Some(f) = Value::as_f64(v) {
                            rules.pricing_adjustment = f;
                        }
                    }
                    "tax_rate" => {
                        if let Some(f) = Value::as_f64(v) {
                            rules.tax_rate = f;
                        }
                    }
                    "booking_fee" => {
                        if let Some(f) = Value::as_f64(v) {
                            rules.booking_fee = f;
                        }
                    }
                    "trip_hold_seconds" => {
                        if let Some(u) = Value::as_u64(v) {
                            rules.trip_hold_seconds = u;
                        }
                    }
                    "seat_hold_seconds" => {
                        if let Some(u) = Value::as_u64(v) {
                            rules.seat_hold_seconds = u;
                        }
                    }
                    "sale_start" => {
                        if let Some(s) = v.as_str() {
                            rules.sale_start = Some(String::from(s));
                        }
                    }
                    "sale_end" => {
                        if let Some(s) = v.as_str() {
                            rules.sale_end = Some(String::from(s));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(rules)
    }
}
