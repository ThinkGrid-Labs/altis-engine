use crate::models::{Order, OrderItem, RevenueStatus, LedgerEntry};
use uuid::Uuid;
use chrono::Utc;

/// Handles financial operations for orders
pub struct FinancialManager {
    // Repository would be injected in a real implementation
}

impl FinancialManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Recognize revenue for a specific item
    /// Returns a LedgerEntry if successful
    pub fn recognize_revenue(
        &self,
        order: &Order,
        item_id: Uuid,
    ) -> Option<LedgerEntry> {
        // Find the item in the order
        let item = order.items.iter().find(|i| i.id == item_id)?;

        // Only recognize if unearned
        if item.revenue_status != RevenueStatus::Unearned {
            return None;
        }

        Some(LedgerEntry {
            id: Uuid::new_v4(),
            order_id: order.id,
            order_item_id: item.id,
            transaction_type: "REVENUE_RECOGNITION".to_string(),
            amount_nuc: item.price_nuc,
            currency: order.currency.clone(),
            description: Some(format!("Revenue recognized for {} ({})", item.name, item.product_type)),
            created_at: Utc::now(),
        })
    }

    /// Calculate settlement report for an airline
    /// (Mock logic: in reality, this would query aggregated data)
    pub fn generate_settlement_report(
        &self,
        airline_id: Uuid,
        orders: &[Order],
    ) -> serde_json::Value {
        let mut total_earned = 0;
        let mut total_unearned = 0;
        let mut item_count = 0;

        for order in orders {
            if Some(airline_id) == order.airline_id {
                for item in &order.items {
                    item_count += 1;
                    match item.revenue_status {
                        RevenueStatus::Earned => total_earned += item.price_nuc,
                        RevenueStatus::Unearned => total_unearned += item.price_nuc,
                        RevenueStatus::Refunded => {}
                    }
                }
            }
        }

        serde_json::json!({
            "airline_id": airline_id,
            "report_date": Utc::now().to_rfc3339(),
            "metrics": {
                "total_earned_nuc": total_earned,
                "total_unearned_nuc": total_unearned,
                "processed_items": item_count
            }
        })
    }
}
