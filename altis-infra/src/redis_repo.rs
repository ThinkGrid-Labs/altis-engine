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
    }

    pub async fn decr_flight_availability(&self, flight_id: &str) -> RedisResult<Option<i64>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("flight:{}:availability", flight_id);
        // Enterprise Upgrade: Use Lua script to ensuring we don't seed negative values on cache miss.
        // If key exists, DECR it. If not, return nil (and let the next Search re-seed it from DB).
        let script = redis::Script::new(r#"
            if redis.call("EXISTS", KEYS[1]) == 1 then
                return redis.call("DECR", KEYS[1])
            else
                return nil
            end
        "#);
        
        script.key(key).invoke_async(&mut conn).await
    }
        pub async fn delete_flight_availability(&self, flight_id: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("flight:{}:availability", flight_id);
        conn.del(key).await
    }
        pub async fn del_trip_key(&self, trip_id: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("trip:{}", trip_id);
        conn.del(key).await
    }
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

