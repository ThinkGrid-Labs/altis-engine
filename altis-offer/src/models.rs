use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Offer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OfferStatus {
    Active,
    Expired,
    Accepted,
    Cancelled,
}

/// An offer presented to the customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: Uuid,
    pub customer_id: Option<String>,
    pub airline_id: Option<Uuid>,
    pub search_context: serde_json::Value,
    pub items: Vec<OfferItem>,
    pub total_nuc: i32,
    pub currency: String,
    pub status: OfferStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl Offer {
    /// Create a new offer with 15-minute expiry
    pub fn new(customer_id: Option<String>, airline_id: Option<Uuid>, search_context: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            customer_id,
            airline_id,
            search_context,
            items: Vec::new(),
            total_nuc: 0,
            currency: "NUC".to_string(),
            status: OfferStatus::Active,
            expires_at: now + chrono::Duration::minutes(15),
            created_at: now,
            metadata: serde_json::json!({}),
        }
    }
    
    /// Add an item to the offer
    pub fn add_item(&mut self, item: OfferItem) {
        self.total_nuc += item.price_nuc;
        self.items.push(item);
    }
    
    /// Check if offer is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Check if offer is still active
    pub fn is_active(&self) -> bool {
        self.status == OfferStatus::Active && !self.is_expired()
    }
}

/// An item within an offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferItem {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub product_type: String,
    pub product_code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub price_nuc: i32,
    pub quantity: i32,
    pub metadata: serde_json::Value,
}

impl OfferItem {
    pub fn new(
        product_type: String,
        product_id: Option<Uuid>,
        product_code: Option<String>,
        name: String,
        description: Option<String>,
        price_nuc: i32,
        quantity: i32,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            product_id,
            product_type,
            product_code,
            name,
            description,
            price_nuc,
            quantity,
            metadata,
        }
    }
}
