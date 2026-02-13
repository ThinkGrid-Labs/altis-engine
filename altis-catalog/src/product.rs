use serde::{Deserialize, Serialize};
use uuid::Uuid;
use async_trait::async_trait;
use crate::pricing::PricingContext;

/// Product types in the catalog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductType {
    Flight,
    Seat,
    Bag,
    Meal,
    Lounge,
    CarbonOffset,
    Insurance,
    FastTrack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlightStatus {
    Scheduled,
    Delayed,
    Cancelled,
    Diverted,
}

/// Core product structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub product_type: ProductType,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_nuc: i32,
    pub margin_percentage: f64,
    pub is_active: bool,
    pub metadata: serde_json::Value,
}

/// Product trait for dynamic pricing
#[async_trait]
pub trait ProductTrait: Send + Sync {
    /// Calculate the current price based on context
    async fn calculate_price(&self, context: &PricingContext) -> Result<i32, ProductError>;
    
    /// Check if product is available
    async fn is_available(&self, context: &PricingContext) -> Result<bool, ProductError>;
    
    /// Get product metadata
    fn get_metadata(&self) -> &serde_json::Value;
}

/// Product-related errors
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    #[error("Product not found: {0}")]
    NotFound(String),
    
    #[error("Product not available: {0}")]
    NotAvailable(String),
    
    #[error("Invalid pricing context: {0}")]
    InvalidContext(String),
    
    #[error("Pricing calculation failed: {0}")]
    PricingFailed(String),
}

/// Flight-specific product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightProduct {
    pub product: Product,
    pub flight_id: Uuid,
    pub origin: String,
    pub destination: String,
    pub departure_time: chrono::DateTime<chrono::Utc>,
    pub arrival_time: chrono::DateTime<chrono::Utc>,
    pub available_seats: i32,
    pub status: FlightStatus,
}

#[async_trait]
impl ProductTrait for FlightProduct {
    async fn calculate_price(&self, context: &PricingContext) -> Result<i32, ProductError> {
        // Base price
        let mut price = self.product.base_price_nuc;
        
        // Apply demand multiplier based on available seats
        let demand_multiplier = if self.available_seats < 10 {
            1.5
        } else if self.available_seats < 50 {
            1.2
        } else {
            1.0
        };
        
        price = (price as f64 * demand_multiplier) as i32;
        
        // Apply time-based pricing
        if let Some(time_multiplier) = context.time_multiplier {
            price = (price as f64 * time_multiplier) as i32;
        }
        
        Ok(price)
    }
    
    async fn is_available(&self, _context: &PricingContext) -> Result<bool, ProductError> {
        Ok(self.product.is_active && self.available_seats > 0)
    }
    
    fn get_metadata(&self) -> &serde_json::Value {
        &self.product.metadata
    }
}

/// Ancillary product (bags, meals, seats, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncillaryProduct {
    pub product: Product,
    pub category: String,
    pub quantity_limit: Option<i32>,
}

#[async_trait]
impl ProductTrait for AncillaryProduct {
    async fn calculate_price(&self, context: &PricingContext) -> Result<i32, ProductError> {
        let mut price = self.product.base_price_nuc;
        
        // Apply bundle discount if part of an offer
        if context.is_bundled {
            price = (price as f64 * 0.9) as i32; // 10% bundle discount
        }
        
        Ok(price)
    }
    
    async fn is_available(&self, _context: &PricingContext) -> Result<bool, ProductError> {
        Ok(self.product.is_active)
    }
    
    fn get_metadata(&self) -> &serde_json::Value {
        &self.product.metadata
    }
}
