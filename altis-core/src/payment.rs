use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    RequiresAction,
    Processing,
    Succeeded,
    Canceled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String, // Provider's ID (e.g., pi_123)
    pub order_id: Uuid,
    pub amount: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub reference: Option<String>,
    pub client_secret: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait PaymentAdapter: Send + Sync {
    /// Create a payment intent with the provider
    async fn create_intent(
        &self,
        order_id: Uuid,
        amount: i32,
        currency: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve intent status
    async fn get_intent(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Capture a previously authorized payment
    async fn capture_payment(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Process a payment (Direct Checkout)
    async fn process_payment(
        &self,
        payment: &PaymentIntent,
    ) -> Result<PaymentStatus, Box<dyn std::error::Error + Send + Sync>>;
}
