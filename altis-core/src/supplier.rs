use async_trait::async_trait;
use uuid::Uuid;
use serde_json::Value;

#[async_trait]
pub trait SupplierClient: Send + Sync {
    /// Sync the status of an order item with the external supplier
    async fn sync_item_status(
        &self,
        order_id: Uuid,
        item_id: Uuid,
        external_reference: &str,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;

    /// Notify supplier of a consumption event (e.g., flight taken)
    async fn notify_consumption(
        &self,
        item_id: Uuid,
        barcode: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
