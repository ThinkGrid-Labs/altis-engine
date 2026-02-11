use uuid::Uuid;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Inventory tracking for products
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub product_id: Uuid,
    pub available_quantity: i32,
    pub total_capacity: i32,
    pub reserved_quantity: i32,
}

/// In-memory inventory manager (will integrate with Redis later)
pub struct InventoryManager {
    inventory: HashMap<Uuid, InventoryItem>,
}

impl InventoryManager {
    pub fn new() -> Self {
        Self {
            inventory: HashMap::new(),
        }
    }
    
    /// Initialize inventory for a product
    pub fn initialize(&mut self, product_id: Uuid, total_capacity: i32) {
        self.inventory.insert(product_id, InventoryItem {
            product_id,
            available_quantity: total_capacity,
            total_capacity,
            reserved_quantity: 0,
        });
    }
    
    /// Get current inventory
    pub fn get(&self, product_id: &Uuid) -> Option<&InventoryItem> {
        self.inventory.get(product_id)
    }
    
    /// Reserve inventory (for offer creation)
    pub fn reserve(&mut self, product_id: &Uuid, quantity: i32) -> Result<(), InventoryError> {
        let item = self.inventory.get_mut(product_id)
            .ok_or_else(|| InventoryError::NotFound(product_id.to_string()))?;
        
        if item.available_quantity < quantity {
            return Err(InventoryError::InsufficientInventory {
                requested: quantity,
                available: item.available_quantity,
            });
        }
        
        item.available_quantity -= quantity;
        item.reserved_quantity += quantity;
        
        Ok(())
    }
    
    /// Release reserved inventory (offer expired)
    pub fn release(&mut self, product_id: &Uuid, quantity: i32) -> Result<(), InventoryError> {
        let item = self.inventory.get_mut(product_id)
            .ok_or_else(|| InventoryError::NotFound(product_id.to_string()))?;
        
        item.available_quantity += quantity;
        item.reserved_quantity = item.reserved_quantity.saturating_sub(quantity);
        
        Ok(())
    }
    
    /// Commit reserved inventory (offer accepted)
    pub fn commit(&mut self, product_id: &Uuid, quantity: i32) -> Result<(), InventoryError> {
        let item = self.inventory.get_mut(product_id)
            .ok_or_else(|| InventoryError::NotFound(product_id.to_string()))?;
        
        if item.reserved_quantity < quantity {
            return Err(InventoryError::InsufficientReserved {
                requested: quantity,
                reserved: item.reserved_quantity,
            });
        }
        
        item.reserved_quantity -= quantity;
        
        Ok(())
    }
    
    /// Get utilization percentage
    pub fn get_utilization(&self, product_id: &Uuid) -> Option<f64> {
        self.inventory.get(product_id).map(|item| {
            if item.total_capacity == 0 {
                0.0
            } else {
                1.0 - (item.available_quantity as f64 / item.total_capacity as f64)
            }
        })
    }
}

impl Default for InventoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("Inventory not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient inventory: requested {requested}, available {available}")]
    InsufficientInventory {
        requested: i32,
        available: i32,
    },
    
    #[error("Insufficient reserved inventory: requested {requested}, reserved {reserved}")]
    InsufficientReserved {
        requested: i32,
        reserved: i32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inventory_lifecycle() {
        let mut manager = InventoryManager::new();
        let product_id = Uuid::new_v4();
        
        // Initialize
        manager.initialize(product_id, 100);
        assert_eq!(manager.get(&product_id).unwrap().available_quantity, 100);
        
        // Reserve
        manager.reserve(&product_id, 10).unwrap();
        assert_eq!(manager.get(&product_id).unwrap().available_quantity, 90);
        assert_eq!(manager.get(&product_id).unwrap().reserved_quantity, 10);
        
        // Commit
        manager.commit(&product_id, 10).unwrap();
        assert_eq!(manager.get(&product_id).unwrap().reserved_quantity, 0);
        
        // Check utilization
        let utilization = manager.get_utilization(&product_id).unwrap();
        assert!((utilization - 0.1).abs() < 0.01); // 10% utilized
    }
}
