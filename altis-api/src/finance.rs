use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerResponse {
    pub order_id: Uuid,
    pub entries: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettlementReportResponse {
    pub airline_id: Uuid,
    pub report_date: String,
    pub metrics: SettlementMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettlementMetrics {
    pub total_earned_nuc: i32,
    pub total_unearned_nuc: i32,
    pub total_payable_nuc: i32,    // Amount owed to suppliers
    pub total_commission_nuc: i32, // Amount kept as retailer
    pub processed_items: i32,
}

/// GET /v1/admin/finance/orders/:id/ledger
pub async fn get_order_ledger(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<LedgerResponse>, StatusCode> {
    let entries = state.order_repo.get_order_ledger(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LedgerResponse {
        order_id,
        entries,
    }))
}

/// GET /v1/admin/finance/airlines/:id/settlement
pub async fn get_airline_settlement(
    State(_state): State<AppState>,
    Path(airline_id): Path<Uuid>,
) -> Result<Json<SettlementReportResponse>, StatusCode> {
    // 1. Fetch all orders for this airline (Mock filtering or repo query)
    // For now, we'll fetch a sample or just return a mock based on the logic
    
    // In a real repo, we'd have state.order_repo.get_airline_summary(airline_id)
    
    Ok(Json(SettlementReportResponse {
        airline_id,
        report_date: chrono::Utc::now().to_rfc3339(),
        metrics: SettlementMetrics {
            total_earned_nuc: 125000,
            total_unearned_nuc: 45000,
            total_payable_nuc: 30000,
            total_commission_nuc: 5000,
            processed_items: 128,
        },
    }))
}

/// GET /v1/admin/finance/airlines/:id/export/swo
pub async fn export_swo(
    State(_state): State<AppState>,
    Path(_airline_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 1. Fetch relevant ledger entries (Mocked for now)
    // 2. Use altis_order::settlement::IataSwoAdaptor to adapt
    
    let order_mock = altis_order::Order::new("test@example.com".to_string());
    let ledger_mock = vec![altis_order::models::LedgerEntry {
        id: Uuid::new_v4(),
        order_id: order_mock.id,
        order_item_id: Uuid::new_v4(),
        transaction_type: "REVENUE_RECOGNITION".to_string(),
        amount_nuc: 5000,
        currency: "NUC".to_string(),
        description: Some("Settlement Export Test".to_string()),
        created_at: chrono::Utc::now(),
    }];

    let adaptor = altis_order::settlement::IataSwoAdaptor;
    let payload = altis_order::settlement::SettlementAdaptor::adapt(&adaptor, &order_mock, ledger_mock).await
        .map_err(|e| {
            tracing::error!("Swo Adaptor failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(payload))
}

/// GET /v1/admin/finance/airlines/:id/export/legacy
pub async fn export_legacy(
    State(_state): State<AppState>,
    Path(_airline_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let order_mock = altis_order::Order::new("test@example.com".to_string());
    let ledger_mock = vec![altis_order::models::LedgerEntry {
        id: Uuid::new_v4(),
        order_id: order_mock.id,
        order_item_id: Uuid::new_v4(),
        transaction_type: "REVENUE_RECOGNITION".to_string(),
        amount_nuc: 5000,
        currency: "NUC".to_string(),
        description: Some("Legacy Export Test".to_string()),
        created_at: chrono::Utc::now(),
    }];

    let adaptor = altis_order::settlement::LegacyHotAdaptor;
    let payload = altis_order::settlement::SettlementAdaptor::adapt(&adaptor, &order_mock, ledger_mock).await
        .map_err(|e| {
            tracing::error!("Legacy Adaptor failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(payload))
}
