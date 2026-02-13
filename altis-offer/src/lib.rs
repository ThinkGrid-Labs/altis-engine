pub mod models;
pub mod generator;
pub mod ai_ranker;
pub mod expiry;
pub mod features;
pub mod events;
pub mod rules;

pub use models::{Offer, OfferItem, OfferStatus};
pub use generator::OfferGenerator;
pub use ai_ranker::OfferRanker;
pub use expiry::ExpiryManager;
