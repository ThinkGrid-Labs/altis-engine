use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SeatHeldEvent {
    pub flight_id: Uuid,
    pub seat_number: String,
    pub trip_id: Uuid,
    pub held_at: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OfferGeneratedEvent {
    pub offer_id: Uuid,
    pub customer_id: Option<String>,
    pub timestamp: i64,
    pub search_context: serde_json::Value,
    pub features: serde_json::Value, // Serialized OfferFeatures
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OfferAcceptedEvent {
    pub offer_id: Uuid,
    pub customer_id: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OrderPaidEvent {
    pub order_id: Uuid,
    pub offer_id: Option<Uuid>,
    pub customer_id: String,
    pub total_nuc: i32,
    pub timestamp: i64,
}
