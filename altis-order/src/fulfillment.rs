use crate::models::{Fulfillment, OrderItem};
use uuid::Uuid;
use std::collections::HashMap;

/// Generates and manages fulfillment records
pub struct FulfillmentService {
    fulfillments: HashMap<Uuid, Fulfillment>,
}

impl FulfillmentService {
    pub fn new() -> Self {
        Self {
            fulfillments: HashMap::new(),
        }
    }
    
    /// Generate fulfillment for an order item
    pub fn generate_fulfillment(&mut self, order_item: &OrderItem) -> Result<Fulfillment, FulfillmentError> {
        let barcode = self.generate_barcode(&order_item.id);
        let fulfillment = Fulfillment::new(order_item.id, barcode);
        
        self.fulfillments.insert(fulfillment.id, fulfillment.clone());
        Ok(fulfillment)
    }
    
    /// Get fulfillment by ID
    pub fn get_fulfillment(&self, fulfillment_id: &Uuid) -> Option<&Fulfillment> {
        self.fulfillments.get(fulfillment_id)
    }
    
    /// Get fulfillment by barcode token
    pub fn get_by_barcode(&self, barcode: &str) -> Option<&Fulfillment> {
        self.fulfillments.values()
            .find(|f| f.barcode_token == barcode)
    }
    
    /// Mark fulfillment as consumed (e.g., boarding pass scanned)
    pub fn consume(&mut self, barcode: &str) -> Result<(), FulfillmentError> {
        let fulfillment = self.fulfillments.values_mut()
            .find(|f| f.barcode_token == barcode)
            .ok_or_else(|| FulfillmentError::NotFound(barcode.to_string()))?;
        
        if fulfillment.is_consumed {
            return Err(FulfillmentError::AlreadyConsumed(barcode.to_string()));
        }
        
        fulfillment.consume();
        Ok(())
    }
    
    /// Generate a unique barcode token
    fn generate_barcode(&self, order_item_id: &Uuid) -> String {
        // Format: ALTIS-{timestamp}-{short_uuid}
        let timestamp = chrono::Utc::now().timestamp();
        let short_id = &order_item_id.to_string()[..8];
        format!("ALTIS-{}-{}", timestamp, short_id.to_uppercase())
    }
    
    /// Generate QR code data (for mobile boarding passes)
    pub fn generate_qr_data(&self, fulfillment: &Fulfillment) -> String {
        // Simple JSON format for QR code
        serde_json::json!({
            "barcode": fulfillment.barcode_token,
            "order_item_id": fulfillment.order_item_id,
            "created_at": fulfillment.created_at,
        }).to_string()
    }
}

impl Default for FulfillmentService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FulfillmentError {
    #[error("Fulfillment not found: {0}")]
    NotFound(String),
    
    #[error("Fulfillment already consumed: {0}")]
    AlreadyConsumed(String),
    
    #[error("Barcode generation failed: {0}")]
    GenerationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OrderItemStatus;
    
    #[test]
    fn test_fulfillment_generation() {
        let mut service = FulfillmentService::new();
        
        let order_item = OrderItem::new(
            Uuid::new_v4(),
            "FLIGHT".to_string(),
            Uuid::new_v4(),
            "Test Flight".to_string(),
            10000,
            serde_json::json!({}),
        );
        
        let fulfillment = service.generate_fulfillment(&order_item).unwrap();
        
        assert!(fulfillment.barcode_token.starts_with("ALTIS-"));
        assert!(!fulfillment.is_consumed);
    }
    
    #[test]
    fn test_barcode_consumption() {
        let mut service = FulfillmentService::new();
        
        let order_item = OrderItem::new(
            Uuid::new_v4(),
            "FLIGHT".to_string(),
            Uuid::new_v4(),
            "Test Flight".to_string(),
            10000,
            serde_json::json!({}),
        );
        
        let fulfillment = service.generate_fulfillment(&order_item).unwrap();
        let barcode = fulfillment.barcode_token.clone();
        
        // First consumption should succeed
        service.consume(&barcode).unwrap();
        
        // Second consumption should fail
        let result = service.consume(&barcode);
        assert!(result.is_err());
    }
}
