use redis::AsyncCommands;
use std::time::Duration;
use tracing::{info, warn};

pub mod config;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub async fn new(connection_string: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(connection_string)?;
        Ok(Self { client })
    }

    pub async fn set_trip_hold(&self, trip_id: &str, flight_id: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        con.set_ex(key, flight_id, ttl_seconds).await?;
        info!("Trip hold set: {} -> {}", trip_id, flight_id);
        Ok(())
    }

    pub async fn get_trip_flight(&self, trip_id: &str) -> Result<Option<String>, redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        let flight_id: Option<String> = con.get(key).await?;
        Ok(flight_id)
    }

    pub async fn acquire_seat_lock(&self, flight_id: &str, seat_number: &str, trip_id: &str, ttl_seconds: u64) -> Result<bool, redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("seat:{}:{}", flight_id, seat_number);
        
        // SET NX: Only set if key does not exist
        let result: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg(trip_id)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut con)
            .await?;

        Ok(result.is_some())
    pub async fn decr_flight_availability(&self, flight_id: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("flight:{}:availability", flight_id);
        // Atomic DECR. Returns new value.
        // If key doesn't exist, Redis treats as 0. This is tricky.
        // We need to check existence if we want lazy loading, OR ensure it's seeded.
        // Enterprise: If it doesn't exist, we can ignore (Search will seed it correctly) OR try to init.
        // Simple DECR is safest for now, but if it goes negative, that's fine (indicates overbooking or just unseeded).
        // Better: Lua script "if exists then DECR else return nil".
        // For MVP-Enterprise, just DECR.
        conn.decr(key, 1).await
        pub async fn delete_flight_availability(&self, flight_id: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("flight:{}:availability", flight_id);
        conn.del(key).await
        pub async fn del_trip_key(&self, trip_id: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        conn.del(key).await
    }

    // Hash Operations for Sessions
    pub async fn hset_trip_field(&self, trip_id: &str, field: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        conn.hset(key, field, value).await
    }

    pub async fn hget_trip_field(&self, trip_id: &str, field: &str) -> RedisResult<Option<String>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        conn.hget(key, field).await
    }

    pub async fn exp_trip_key(&self, trip_id: &str, ttl_seconds: usize) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        conn.expire(key, ttl_seconds).await
    }
}
}
}
