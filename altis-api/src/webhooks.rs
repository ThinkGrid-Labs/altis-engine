use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::state::AppState;
use altis_core::payment::PaymentStatus;

#[derive(Debug, Deserialize)]
pub struct StripeWebhook {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: WebhookData,
}

#[derive(Debug, Deserialize)]
pub struct WebhookData {
    pub object: PaymentIntentObject,
}

#[derive(Debug, Deserialize)]
pub struct PaymentIntentObject {
    pub id: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}

/// POST /v1/webhooks/payments/stripe
/// Receive payment status updates from Stripe
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    Json(payload): Json<StripeWebhook>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Received webhook: {} for intent {}", payload.type_, payload.data.object.id);

    if payload.type_ == "payment_intent.succeeded" {
        let intent_id = &payload.data.object.id;
        
        // 1. Process status update via orchestrator
        let intent = state.payment_orchestrator.process_status_update(intent_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if intent.status == PaymentStatus::Succeeded {
            // 2. Mark order as PAID
            state.order_repo.update_order_status(intent.order_id, "PAID").await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            tracing::info!("Order {} marked as PAID via webhook", intent.order_id);
        }
    }

    Ok(StatusCode::OK)
}
