use async_trait::async_trait;
use crate::search::{FlightOption, SearchLeg};
use std::error::Error;

#[async_trait]
pub trait FlightRepository: Send + Sync {
    async fn search_flights(
        &self,
        leg: &SearchLeg,
        min_seats: u32,
    ) -> Result<Vec<FlightOption>, Box<dyn Error + Send + Sync>>;
}
