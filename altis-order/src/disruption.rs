use crate::models::{Order, OrderItem, OrderItemStatus};
use altis_catalog::product::{FlightProduct, FlightStatus};
use uuid::Uuid;

/// Result of a re-accommodation attempt
pub struct ReaccommodationResult {
    pub order_id: Uuid,
    pub protected_items: Vec<Uuid>,
    pub new_proposed_items: Vec<OrderItem>,
}

/// Handles the logic for identifying and resolving flight disruptions
pub struct DisruptionManager {
    // In a real system, this would have access to repositories
}

impl DisruptionManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Process a flight status change and identify affected orders
    /// (In this mock implementation, we take the orders as input)
    pub fn process_disruption(
        &self,
        flight_id: Uuid,
        new_status: FlightStatus,
        affected_orders: &mut [Order],
        alternatives: &[FlightProduct],
    ) -> Vec<ReaccommodationResult> {
        let mut results = Vec::new();

        if new_status == FlightStatus::Cancelled || new_status == FlightStatus::Delayed {
            for order in affected_orders {
                let mut protected_items = Vec::new();
                let mut proposed_items = Vec::new();

                // 1. Identify and Protect affected items
                for item in &mut order.items {
                    if item.product_type == "FLIGHT" {
                        if let Some(item_flight_id) = item.metadata.get("flight_id").and_then(|id| id.as_str()) {
                            if item_flight_id == flight_id.to_string() {
                                item.status = OrderItemStatus::Protected;
                                protected_items.push(item.id);
                            }
                        }
                    }
                }

                // 2. Search for Re-accommodation (if cancellation)
                if new_status == FlightStatus::Cancelled && !protected_items.is_empty() {
                    // Primitive re-accommodation logic: take the first available alternative
                    if let Some(alt_flight) = alternatives.first() {
                        let mut metadata = alt_flight.product.metadata.clone();
                        metadata["flight_id"] = serde_json::json!(alt_flight.flight_id.to_string());
                        metadata["disrupted_item_id"] = serde_json::json!(protected_items[0].to_string());

                        let new_item = OrderItem::new(
                            "FLIGHT".to_string(),
                            Some(alt_flight.product.id),
                            Some(alt_flight.product.product_code.clone()),
                            alt_flight.product.name.clone(),
                            alt_flight.product.description.clone(),
                            0, // Involuntary re-accommodation is usually zero-cost to customer
                            1,
                            metadata,
                        );
                        
                        // Set status to Reaccommodated (pending acceptance)
                        let mut reac_item = new_item;
                        reac_item.status = OrderItemStatus::Reaccommodated;
                        proposed_items.push(reac_item);
                    }
                }

                if !protected_items.is_empty() {
                    results.push(ReaccommodationResult {
                        order_id: order.id,
                        protected_items,
                        new_proposed_items: proposed_items,
                    });
                }
            }
        }

        results
    }
}
