use crate::models::{Order, OrderItem, OrderItemStatus};
use uuid::Uuid;

/// Handles order modifications and changes
pub struct ChangeHandler;

impl ChangeHandler {
    /// Add a new item to an existing order
    pub fn add_item(order: &mut Order, new_item: OrderItem) -> Result<(), ChangeError> {
        // Validate order is in a modifiable state
        if !Self::is_modifiable(order) {
            return Err(ChangeError::OrderNotModifiable(order.id.to_string()));
        }
        
        order.add_item(new_item);
        Ok(())
    }
    
    /// Refund an item (mark as REFUNDED, never delete)
    pub fn refund_item(order: &mut Order, item_id: &Uuid) -> Result<(), ChangeError> {
        let item = order.items.iter_mut()
            .find(|i| i.id == *item_id)
            .ok_or_else(|| ChangeError::ItemNotFound(item_id.to_string()))?;
        
        if item.status != OrderItemStatus::Active {
            return Err(ChangeError::ItemNotActive(item_id.to_string()));
        }
        
        item.refund();
        
        // Recalculate order total
        order.total_nuc = order.calculate_active_total();
        order.updated_at = chrono::Utc::now();
        
        Ok(())
    }
    
    /// Change flight (add new flight item, refund old one)
    pub fn change_flight(
        order: &mut Order,
        old_flight_item_id: &Uuid,
        new_flight_item: OrderItem,
    ) -> Result<(), ChangeError> {
        // Refund old flight
        Self::refund_item(order, old_flight_item_id)?;
        
        // Add new flight
        Self::add_item(order, new_flight_item)?;
        
        Ok(())
    }
    
    /// Check if order can be modified
    fn is_modifiable(order: &Order) -> bool {
        use crate::models::OrderStatus;
        matches!(order.status, OrderStatus::Proposed | OrderStatus::Locked | OrderStatus::Paid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChangeError {
    #[error("Order not modifiable: {0}")]
    OrderNotModifiable(String),
    
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    #[error("Item not active: {0}")]
    ItemNotActive(String),
    
    #[error("Change validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OrderStatus;
    
    #[test]
    fn test_add_item() {
        let mut order = Order::new("customer@example.com".to_string());
        
        let new_item = OrderItem::new(
            order.id,
            "MEAL".to_string(),
            Uuid::new_v4(),
            "Vegetarian Meal".to_string(),
            1500,
            serde_json::json!({}),
        );
        
        let initial_total = order.total_nuc;
        ChangeHandler::add_item(&mut order, new_item).unwrap();
        
        assert_eq!(order.items.len(), 1);
        assert_eq!(order.total_nuc, initial_total + 1500);
    }
    
    #[test]
    fn test_refund_item() {
        let mut order = Order::new("customer@example.com".to_string());
        
        let item = OrderItem::new(
            order.id,
            "BAG".to_string(),
            Uuid::new_v4(),
            "Extra Bag".to_string(),
            3000,
            serde_json::json!({}),
        );
        let item_id = item.id;
        
        order.add_item(item);
        let initial_total = order.total_nuc;
        
        ChangeHandler::refund_item(&mut order, &item_id).unwrap();
        
        assert_eq!(order.items[0].status, OrderItemStatus::Refunded);
        assert_eq!(order.total_nuc, 0); // Refunded items don't count
    }
    
    #[test]
    fn test_change_flight() {
        let mut order = Order::new("customer@example.com".to_string());
        
        let old_flight = OrderItem::new(
            order.id,
            "FLIGHT".to_string(),
            Uuid::new_v4(),
            "Old Flight".to_string(),
            20000,
            serde_json::json!({}),
        );
        let old_flight_id = old_flight.id;
        order.add_item(old_flight);
        
        let new_flight = OrderItem::new(
            order.id,
            "FLIGHT".to_string(),
            Uuid::new_v4(),
            "New Flight".to_string(),
            25000,
            serde_json::json!({}),
        );
        
        ChangeHandler::change_flight(&mut order, &old_flight_id, new_flight).unwrap();
        
        assert_eq!(order.items.len(), 2);
        assert_eq!(order.items[0].status, OrderItemStatus::Refunded);
        assert_eq!(order.items[1].status, OrderItemStatus::Active);
        assert_eq!(order.total_nuc, 25000);
    }
}
