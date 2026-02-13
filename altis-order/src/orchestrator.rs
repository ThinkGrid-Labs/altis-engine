use altis_core::payment::{PaymentAdapter, PaymentIntent, PaymentStatus};
use uuid::Uuid;
use std::sync::Arc;

pub struct PaymentOrchestrator {
    adapter: Arc<dyn PaymentAdapter>,
}

impl PaymentOrchestrator {
    pub fn new(adapter: Arc<dyn PaymentAdapter>) -> Self {
        Self { adapter }
    }

    /// Initialize a payment intent for an order
    pub async fn initialize_payment(
        &self,
        order_id: Uuid,
        amount: i32,
        currency: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>> {
        // Here we could add logic to select different adapters based on currency/country
        self.adapter.create_intent(order_id, amount, currency).await
    }

    /// Process a status update (e.g., from a webhook)
    pub async fn process_status_update(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>> {
        let intent = self.adapter.get_intent(intent_id).await?;
        
        if intent.status == PaymentStatus::Succeeded {
            // In a real system, we might trigger capture here if it's an Auth-Only flow
        }
        
        Ok(intent)
    }

    pub async fn process_payment(
        &self,
        payment: &altis_core::payment::PaymentIntent,
    ) -> Result<altis_core::payment::PaymentStatus, Box<dyn std::error::Error + Send + Sync>> {
        self.adapter.process_payment(payment).await
    }
}

pub struct MockPaymentAdapter;

#[async_trait::async_trait]
impl PaymentAdapter for MockPaymentAdapter {
    async fn create_intent(
        &self,
        order_id: Uuid,
        amount: i32,
        currency: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(PaymentIntent {
            // Encode order_id in intent_id for the mock to "remember" it
            id: format!("mock_pi_{}", order_id.simple()),
            order_id,
            amount,
            currency: currency.to_string(),
            status: PaymentStatus::RequiresPaymentMethod,
            reference: None,
            client_secret: Some("mock_secret_123".to_string()),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_intent(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>> {
        // Decode order_id from mock intent_id
        let order_id_str = intent_id.strip_prefix("mock_pi_").unwrap_or_default();
        let order_id = Uuid::parse_str(order_id_str).unwrap_or_else(|_| Uuid::new_v4());

        // Return a dummy intent that is "Succeeded" to simulate successful payment
        Ok(PaymentIntent {
            id: intent_id.to_string(),
            order_id,
            amount: 1000,
            currency: "NUC".to_string(),
            status: PaymentStatus::Succeeded,
            reference: None,
            client_secret: None,
            created_at: chrono::Utc::now(),
        })
    }

    async fn capture_payment(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent, Box<dyn std::error::Error + Send + Sync>> {
        self.get_intent(intent_id).await
    }

    async fn process_payment(&self, payment: &PaymentIntent) -> Result<PaymentStatus, Box<dyn std::error::Error + Send + Sync>> {
        // Trigger for testing Circuit Breaker
        if payment.reference.as_deref() == Some("fail-circuit") {
            return Err("Simulated Payment Gateway Failure".into());
        }
        Ok(PaymentStatus::Succeeded)
    }
}
