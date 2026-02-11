pub mod app_config;
pub mod database;
pub mod redis_repo;
pub mod events;
pub mod flight_repo;
pub mod offer_repo;
pub mod order_repo;

pub use database::DbClient;
pub use redis_repo::RedisClient;
pub use events::EventProducer;
pub use flight_repo::PostgresFlightRepository;
pub use offer_repo::PostgresOfferRepository;
pub use order_repo::PostgresOrderRepository;

pub fn hello() {
    println!("Hello from Altis Store!");
}
