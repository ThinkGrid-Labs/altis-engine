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
    pub search_context: serde_json::Value,
    pub items: Vec<OfferItem>,
    pub total_nuc: i32,
    pub currency: String,
    pub status: OfferStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Offer {
    /// Create a new offer with 15-minute expiry
    pub fn new(customer_id: Option<String>, search_context: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            customer_id,
            search_context,
            items: Vec::new(),
            total_nuc: 0,
            currency: "NUC".to_string(),
            status: OfferStatus::Active,
            expires_at: now + chrono::Duration::minutes(15),
            created_at: now,
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
    pub offer_id: Uuid,
    pub product_type: String,
    pub product_id: Uuid,
    pub product_name: String,
    pub price_nuc: i32,
    pub metadata: serde_json::Value,
}

impl OfferItem {
    pub fn new(
        offer_id: Uuid,
        product_type: String,
        product_id: Uuid,
        product_name: String,
        price_nuc: i32,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            offer_id,
            product_type,
            product_id,
            product_name,
            price_nuc,
            metadata,
        }
    }
}
