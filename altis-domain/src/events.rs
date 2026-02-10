use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeatHeldEvent {
    pub flight_id: Uuid,
    pub seat_number: String,
    pub trip_id: Uuid,
    pub held_at: i64,
}
