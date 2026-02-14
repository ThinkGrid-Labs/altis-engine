use async_trait::async_trait;
use uuid::Uuid;


use crate::search::FlightSearchResult;

// Re-export types from other crates to avoid circular dependencies
// These are defined here as traits/interfaces that implementations will use

/// Repository trait for flight data access
#[async_trait]
pub trait FlightRepository: Send + Sync {
    async fn search_flights(
        &self,
        origin: &str,
        destination: &str,
        date: &str,
    ) -> Result<Vec<FlightSearchResult>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Generic repository trait for offer data access
/// Uses serde_json::Value to avoid circular dependencies
#[async_trait]
pub trait OfferRepository: Send + Sync {
    async fn save_offer(
        &self,
        offer: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_offer(
        &self,
        id: Uuid,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_active_offers(
        &self,
        customer_id: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn expire_offer(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Generic repository trait for order data access
#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create_order(
        &self,
        order: &serde_json::Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_order(
        &self,
        id: Uuid,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn update_order_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: &serde_json::Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_orders(
        &self,
        customer_id: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;

    async fn create_fulfillment(
        &self,
        order_id: Uuid,
        order_item_id: Uuid,
        fulfillment_type: &str,
        barcode: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;

    async fn consume_fulfillment(
        &self,
        barcode: &str,
        location: &str,
    ) -> Result<(Uuid, Uuid), Box<dyn std::error::Error + Send + Sync>>;

    async fn add_order_change(
        &self,
        order_id: Uuid,
        change_type: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn find_orders_by_flight(
        &self,
        flight_id: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;

    async fn add_order_ledger_entry(
        &self,
        order_id: Uuid,
        order_item_id: Uuid,
        transaction_type: &str,
        amount_nuc: i32,
        description: Option<&str>,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;

    async fn update_item_revenue_status(
        &self,
        item_id: Uuid,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn get_order_ledger(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Generic repository trait for product catalog access
#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create_product(
        &self,
        product: &serde_json::Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_product(
        &self,
        id: Uuid,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_products(
        &self,
        airline_id: Uuid,
        product_type: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn update_product(
        &self,
        id: Uuid,
        product: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn delete_product(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn get_airline_by_code(
        &self,
        code: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_inventory_rule(
        &self,
        airline_id: Uuid,
        resource_type: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
}
