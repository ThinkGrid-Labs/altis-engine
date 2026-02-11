use crate::models::{Order, OrderStatus, OrderItem};
use uuid::Uuid;
use std::collections::HashMap;

/// Manages order lifecycle and state transitions
pub struct OrderManager {
    orders: HashMap<Uuid, Order>,
}

impl OrderManager {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }
    
    /// Create a new order from an accepted offer
    pub fn create_order(&mut self, customer_id: String, items: Vec<OrderItem>) -> Result<Order, OrderError> {
        let mut order = Order::new(customer_id);
        
        for item in items {
            order.add_item(item);
        }
        
        self.orders.insert(order.id, order.clone());
        Ok(order)
    }
    
    /// Get an order by ID
    pub fn get_order(&self, order_id: &Uuid) -> Option<&Order> {
        self.orders.get(order_id)
    }
    
    /// Transition: Proposed → Locked (inventory reserved)
    pub fn lock_order(&mut self, order_id: &Uuid) -> Result<(), OrderError> {
        let order = self.get_order_mut(order_id)?;
        
        if order.status != OrderStatus::Proposed {
            return Err(OrderError::InvalidTransition {
                from: format!("{:?}", order.status),
                to: "LOCKED".to_string(),
            });
        }
        
        order.update_status(OrderStatus::Locked);
        Ok(())
    }
    
    /// Transition: Locked → Paid (payment confirmed)
    pub fn mark_paid(&mut self, order_id: &Uuid) -> Result<(), OrderError> {
        let order = self.get_order_mut(order_id)?;
        
        if order.status != OrderStatus::Locked {
            return Err(OrderError::InvalidTransition {
                from: format!("{:?}", order.status),
                to: "PAID".to_string(),
            });
        }
        
        order.update_status(OrderStatus::Paid);
        Ok(())
    }
    
    /// Transition: Paid → Fulfilled (all items delivered)
    pub fn mark_fulfilled(&mut self, order_id: &Uuid) -> Result<(), OrderError> {
        let order = self.get_order_mut(order_id)?;
        
        if order.status != OrderStatus::Paid {
            return Err(OrderError::InvalidTransition {
                from: format!("{:?}", order.status),
                to: "FULFILLED".to_string(),
            });
        }
        
        order.update_status(OrderStatus::Fulfilled);
        Ok(())
    }
    
    /// Cancel an order (any status except Fulfilled/Archived)
    pub fn cancel_order(&mut self, order_id: &Uuid) -> Result<(), OrderError> {
        let order = self.get_order_mut(order_id)?;
        
        if matches!(order.status, OrderStatus::Fulfilled | OrderStatus::Archived) {
            return Err(OrderError::InvalidTransition {
                from: format!("{:?}", order.status),
                to: "CANCELLED".to_string(),
            });
        }
        
        order.update_status(OrderStatus::Cancelled);
        Ok(())
    }
    
    /// Archive an order (final state)
    pub fn archive_order(&mut self, order_id: &Uuid) -> Result<(), OrderError> {
        let order = self.get_order_mut(order_id)?;
        
        if order.status != OrderStatus::Fulfilled {
            return Err(OrderError::InvalidTransition {
                from: format!("{:?}", order.status),
                to: "ARCHIVED".to_string(),
            });
        }
        
        order.update_status(OrderStatus::Archived);
        Ok(())
    }
    
    /// Helper to get mutable order reference
    fn get_order_mut(&mut self, order_id: &Uuid) -> Result<&mut Order, OrderError> {
        self.orders.get_mut(order_id)
            .ok_or_else(|| OrderError::NotFound(order_id.to_string()))
    }
}

impl Default for OrderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("Order not found: {0}")]
    NotFound(String),
    
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition {
        from: String,
        to: String,
    },
    
    #[error("Order modification failed: {0}")]
    ModificationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_lifecycle() {
        let mut manager = OrderManager::new();
        
        // Create order
        let order = manager.create_order("customer@example.com".to_string(), vec![]).unwrap();
        let order_id = order.id;
        
        // Proposed → Locked
        manager.lock_order(&order_id).unwrap();
        assert_eq!(manager.get_order(&order_id).unwrap().status, OrderStatus::Locked);
        
        // Locked → Paid
        manager.mark_paid(&order_id).unwrap();
        assert_eq!(manager.get_order(&order_id).unwrap().status, OrderStatus::Paid);
        
        // Paid → Fulfilled
        manager.mark_fulfilled(&order_id).unwrap();
        assert_eq!(manager.get_order(&order_id).unwrap().status, OrderStatus::Fulfilled);
        
        // Fulfilled → Archived
        manager.archive_order(&order_id).unwrap();
        assert_eq!(manager.get_order(&order_id).unwrap().status, OrderStatus::Archived);
    }
    
    #[test]
    fn test_invalid_transition() {
        let mut manager = OrderManager::new();
        let order = manager.create_order("customer@example.com".to_string(), vec![]).unwrap();
        let order_id = order.id;
        
        // Cannot go directly from Proposed to Paid
        let result = manager.mark_paid(&order_id);
        assert!(result.is_err());
    }
}
