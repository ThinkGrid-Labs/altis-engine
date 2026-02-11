pub mod app_config;
pub mod redis_repo;
pub mod events;

pub use redis_repo::RedisClient;
pub use events::EventProducer;

pub fn hello() {
    println!("Hello from Altis Store!");
}
