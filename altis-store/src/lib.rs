pub mod app_config;
pub mod redis_repo;
pub mod events;
pub mod offer_repo;
pub mod order_repo;
pub mod catalog_repo;

// Re-export specific structs for easier access
pub use redis_repo::RedisClient;
pub use events::EventProducer;
pub use offer_repo::StoreOfferRepository;
pub use order_repo::StoreOrderRepository;
pub use catalog_repo::StoreProductRepository;
