use axum::{
    extract::State,
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

    if payload.type_ == "payment_intent.succeeded" || payload.type_ == "payment_intent.payment_failed" || payload.type_ == "payment_intent.canceled" {
        let intent_id = &payload.data.object.id;
        
        // 1. Process status update via orchestrator
        let intent = state.payment_orchestrator.process_status_update(intent_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if intent.status == PaymentStatus::Succeeded {
            // 2. Mark order as PAID
            state.order_repo.update_order_status(intent.order_id, "PAID").await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            tracing::info!("Order {} marked as PAID via webhook", intent.order_id);
        } else if intent.status == PaymentStatus::Failed || intent.status == PaymentStatus::Canceled {
            // 2. Mark order as CANCELLED and release inventory
            state.order_repo.update_order_status(intent.order_id, "CANCELLED").await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // 3. Release inventory (Reuse cancellation logic)
            if let Ok(Some(order_json)) = state.order_repo.get_order(intent.order_id).await {
                if let Ok(order) = serde_json::from_value::<crate::orders::OrderResponse>(order_json) {
                    for item in &order.items {
                        if item.product_type == "Flight" {
                            if let Some(product_id) = item.product_id {
                                let pid_str = product_id.to_string();
                                let current = state.redis.get_flight_availability(&pid_str).await
                                    .unwrap_or(Some(0))
                                    .unwrap_or(0);
                                let _ = state.redis.set_flight_availability(&pid_str, current + 1).await;
                            }
                        }
                    }
                }
            }
            
            tracing::info!("Order {} marked as CANCELLED and inventory released via webhook due to payment {:?}", intent.order_id, intent.status);
        }
    }

    Ok(StatusCode::OK)
}
