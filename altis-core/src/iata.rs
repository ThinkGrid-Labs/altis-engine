use serde::{Deserialize, Serialize};

// ============================================================================
// NDC AirShopping Models (v21.3 inspired)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AirShoppingRequest {
    pub party: Party,
    pub shopping_criteria: ShoppingCriteria,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    pub sender: Sender,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sender {
    pub travel_agency: Option<TravelAgency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TravelAgency {
    pub agency_id: String,
    pub iata_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShoppingCriteria {
    pub origin: String,
    pub destination: String,
    pub travel_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirShoppingResponse {
    pub response_id: String,
    pub offers: Vec<NdcOffer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NdcOffer {
    pub offer_id: String,
    pub owner: String, // Airline Code
    pub total_price: NdcPrice,
    pub items: Vec<NdcOfferItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NdcPrice {
    pub amount: i32,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NdcOfferItem {
    pub item_id: String,
    pub service_name: String,
    pub price: NdcPrice,
}

// ============================================================================
// ONE Order Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRetrieveRequest {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneOrderResponse {
    pub order: OneOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneOrder {
    pub order_id: String,
    pub external_id: Option<String>,
    pub status: String,
    pub total_amount: NdcPrice,
    pub order_items: Vec<OneOrderItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneOrderItem {
    pub item_id: String,
    pub product_name: String,
    pub status: String,
    pub price: NdcPrice,
}
