use async_trait::async_trait;
use uuid::Uuid;
use crate::search::FlightSearchResult;

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

/// Repository trait for offer data access
#[async_trait]
pub trait OfferRepository: Send + Sync {
    async fn save_offer(
        &self,
        offer: &altis_offer::Offer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_offer(
        &self,
        id: Uuid,
    ) -> Result<Option<altis_offer::Offer>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_active_offers(
        &self,
        customer_id: &str,
    ) -> Result<Vec<altis_offer::Offer>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn expire_offer(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Repository trait for order data access
#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create_order(
        &self,
        order: &altis_order::Order,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_order(
        &self,
        id: Uuid,
    ) -> Result<Option<altis_order::Order>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn update_order_status(
        &self,
        id: Uuid,
        status: altis_order::OrderStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: &altis_order::OrderItem,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_orders(
        &self,
        customer_id: &str,
    ) -> Result<Vec<altis_order::Order>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Repository trait for product catalog access
#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create_product(
        &self,
        product: &altis_catalog::Product,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_product(
        &self,
        id: Uuid,
    ) -> Result<Option<altis_catalog::Product>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn list_products(
        &self,
        airline_id: Uuid,
        product_type: Option<&str>,
    ) -> Result<Vec<altis_catalog::Product>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn update_product(
        &self,
        id: Uuid,
        product: &altis_catalog::Product,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn delete_product(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
