use crate::models::Offer;
use chrono::{DateTime, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchContext {
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
    pub passengers: i32,
    pub cabin_class: Option<String>,
    pub user_segment: Option<String>,
}

pub struct OfferFeatures {
    // Temporal features
    pub days_until_departure: i32,
    pub is_weekend: bool,
    pub hour_of_day: u32,
    
    // Contextual features
    pub is_domestic: bool,
    pub passenger_count: i32,
    
    // Price features
    pub price_per_passenger: f64,
    pub item_count: i32,
}

impl OfferFeatures {
    pub fn extract(context: &SearchContext, offer: &Offer) -> Self {
        let now = Utc::now();
        
        // 1. Temporal features
        let dep_date = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", context.departure_date))
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);
            
        let days_until_departure = (dep_date - now).num_days() as i32;
        let is_weekend = dep_date.weekday().number_from_monday() > 5;
        let hour_of_day = now.hour();
        
        // 2. Contextual
        let is_domestic = context.origin.len() == 3 && context.destination.len() == 3; // Simplified
        let passenger_count = context.passengers;
        
        // 3. Price
        let item_count = offer.items.len() as i32;
        let price_per_passenger = if passenger_count > 0 {
            offer.total_nuc as f64 / passenger_count as f64
        } else {
            offer.total_nuc as f64
        };

        Self {
            days_until_departure,
            is_weekend,
            hour_of_day,
            is_domestic,
            passenger_count,
            price_per_passenger,
            item_count,
        }
    }
}
