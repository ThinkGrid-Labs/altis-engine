use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    pub id: Uuid,
    pub flight_id: Uuid,
    pub user_email: String,
    pub status: BookingStatus,
    pub total_price_amount: i32,
    pub total_price_currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum BookingStatus {
    PENDING,
    CONFIRMED,
    CANCELLED,
    EXPIRED,
}

impl ToString for BookingStatus {
    fn to_string(&self) -> String {
        match self {
            BookingStatus::PENDING => "PENDING".to_string(),
            BookingStatus::CONFIRMED => "CONFIRMED".to_string(),
            BookingStatus::CANCELLED => "CANCELLED".to_string(),
            BookingStatus::EXPIRED => "EXPIRED".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Passenger {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    // seat_number removed, using separate relation
    pub seats: Vec<PassengerSeat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PassengerSeat {
    pub flight_id: Uuid,
    pub seat_number: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub trip_id: Uuid, // To validate hold
    pub user_email: String,
    pub passengers: Vec<PassengerLoader>,
    pub payment_token: String, // Mock
}

#[derive(Debug, Deserialize)]
pub struct PassengerLoader {
    pub first_name: String,
    pub last_name: String,
    // Map flight_id -> seat_number
    pub seats: Vec<PassengerSeat>,
}
