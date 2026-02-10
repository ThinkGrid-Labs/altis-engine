use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TripHold {
    pub id: Uuid,
    pub flight_id: Uuid,
    pub user_id: String, // Could be email or session ID
    pub created_at: i64, // Unix timestamp
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeatHold {
    pub trip_id: Uuid,
    pub flight_id: Uuid,
    pub seat_number: String,
    pub held_at: i64,
}
