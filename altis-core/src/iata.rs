use serde::{Deserialize, Serialize};

// ============================================================================
// NDC AirShopping Models (v21.3 inspired)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AirShoppingRequest {
    pub party: Party,
    pub shopping_criteria: ShoppingCriteria,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Party {
    pub sender: Sender,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sender {
    pub travel_agency: Option<TravelAgency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TravelAgency {
    pub agency_id: String,
    pub iata_number: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShoppingCriteria {
    pub origin: String,
    pub destination: String,
    pub travel_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AirShoppingResponse {
    pub response_id: String,
    pub offers: Vec<NdcOffer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NdcOffer {
    pub offer_id: String,
    pub owner: String, // Airline Code
    pub total_price: NdcPrice,
    pub items: Vec<NdcOfferItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NdcPrice {
    pub amount: i32,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NdcOfferItem {
    pub item_id: String,
    pub service_name: String,
    pub price: NdcPrice,
}

// ============================================================================
// ONE Order Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderRetrieveRequest {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneOrderResponse {
    pub order: OneOrder,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneOrder {
    pub order_id: String,
    pub external_id: Option<String>,
    pub status: String,
    pub total_amount: NdcPrice,
    pub order_items: Vec<OneOrderItem>,
    pub travelers: Option<Vec<Traveler>>,
    pub contact_info: Option<ContactInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneOrderItem {
    pub item_id: String,
    pub product_name: String,
    pub status: String,
    pub price: NdcPrice,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Traveler {
    pub id: Option<Uuid>,
    pub traveler_index: i32,
    pub ptc: String, // ADT, CHD, etc.
    pub first_name: altis_shared::pii::Masked<String>,
    pub last_name: altis_shared::pii::Masked<String>,
    pub date_of_birth: Option<altis_shared::pii::Masked<String>>, // ISO date
    pub gender: Option<String>,
    pub traveler_did: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactInfo {
    pub email: altis_shared::pii::Masked<String>,
    pub phone: Option<altis_shared::pii::Masked<String>>,
    pub first_name: Option<altis_shared::pii::Masked<String>>,
    pub last_name: Option<altis_shared::pii::Masked<String>>,
}

use uuid::Uuid;
