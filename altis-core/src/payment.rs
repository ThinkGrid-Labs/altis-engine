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

/// Standardized adapter for external payment providers (e.g., Stripe, IATA Pay).
/// This trait allows the Altis Engine to remain provider-agnostic.
#[async_trait]
pub trait PaymentAdapter: Send + Sync {
    /// Create a payment intent with the provider. 
    /// Returns a [PaymentIntent] containing the provider's transaction ID and client secrets.
    async fn create_intent(
        &self,
        order_id: Uuid,
        amount: i32,
        currency: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve the current status of an existing payment intent.
    async fn get_intent(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Capture a previously authorized payment (Auth-Capture flow).
    async fn capture_payment(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>>;

    /// Process a direct payment (Instant Checkout).
    /// Used for synchronous payment processing where the frontend provides a payment token.
    async fn process_payment(
        &self,
        payment: &PaymentIntent,
    ) -> Result<PaymentStatus, Box<dyn std::error::Error + Send + Sync>>;
}
