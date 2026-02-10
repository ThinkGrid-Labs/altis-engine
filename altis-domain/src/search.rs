use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct FlightSearchRequest {
    pub legs: Vec<SearchLeg>,
    pub passenger_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct SearchLeg {
    pub origin_airport_code: String,
    pub destination_airport_code: String,
    pub date: chrono::NaiveDate, // Just date, ignore time for search match
}

#[derive(Debug, Serialize)]
pub struct FlightSearchResult {
    pub legs: Vec<Vec<FlightOption>>,
}

#[derive(Debug, Serialize)]
pub struct FlightOption {
    pub flight_id: Uuid,
    pub flight_number: String,
    pub departure_time: DateTime<Utc>,
    pub arrival_time: DateTime<Utc>,
    pub origin: String,
    pub destination: String,
    pub aircraft_model: String,
    pub remaining_seats: i32,
    pub price_amount: i32,
    pub price_currency: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_search_leg_deserialization() {
        let json = r#"
            {
                "origin_airport_code": "JFK",
                "destination_airport_code": "LHR",
                "date": "2024-12-25"
            }
        "#;
        let leg: SearchLeg = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(leg.origin_airport_code, "JFK");
        assert_eq!(leg.date, NaiveDate::from_ymd_opt(2024, 12, 25).unwrap());
    }
}
