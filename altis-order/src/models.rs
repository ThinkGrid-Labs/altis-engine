use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Order status in the lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Proposed,
    Locked,
    Paid,
    Fulfilled,
    Archived,
    Expired,
    Cancelled,
}

/// Order item status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderItemStatus {
    Active,
    Refunded,
    Cancelled,
    Modified,
}

/// The single source of truth for a customer's purchase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub total_nuc: i32,
    pub currency: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(customer_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            customer_id,
            items: Vec::new(),
            total_nuc: 0,
            currency: "NUC".to_string(),
            status: OrderStatus::Proposed,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add an item to the order
    pub fn add_item(&mut self, item: OrderItem) {
        self.total_nuc += item.price_nuc;
        self.items.push(item);
        self.updated_at = Utc::now();
    }
    
    /// Update order status
    pub fn update_status(&mut self, new_status: OrderStatus) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }
    
    /// Calculate active items total
    pub fn calculate_active_total(&self) -> i32 {
        self.items.iter()
            .filter(|item| item.status == OrderItemStatus::Active)
            .map(|item| item.price_nuc)
            .sum()
    }
}

/// An individual product within an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_type: String,
    pub product_id: Uuid,
    pub product_name: String,
    pub price_nuc: i32,
    pub status: OrderItemStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl OrderItem {
    pub fn new(
        order_id: Uuid,
        product_type: String,
        product_id: Uuid,
        product_name: String,
        price_nuc: i32,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_id,
            product_type,
            product_id,
            product_name,
            price_nuc,
            status: OrderItemStatus::Active,
            metadata,
            created_at: Utc::now(),
        }
    }
    
    /// Mark item as refunded (never delete)
    pub fn refund(&mut self) {
        self.status = OrderItemStatus::Refunded;
    }
}

/// Fulfillment record for delivering order items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fulfillment {
    pub id: Uuid,
    pub order_item_id: Uuid,
    pub barcode_token: String,
    pub is_consumed: bool,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Fulfillment {
    pub fn new(order_item_id: Uuid, barcode_token: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_item_id,
            barcode_token,
            is_consumed: false,
            consumed_at: None,
            created_at: Utc::now(),
        }
    }
    
    /// Mark as consumed (e.g., boarding pass scanned)
    pub fn consume(&mut self) {
        self.is_consumed = true;
        self.consumed_at = Some(Utc::now());
    }
}
