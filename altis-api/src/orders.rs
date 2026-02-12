use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub customer_id: String,
    pub customer_email: Option<String>,
    pub status: String,
    pub items: Vec<OrderItemResponse>,
    pub total_nuc: i32,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_type: String,
    pub name: String,
    pub price_nuc: i32,
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PayOrderRequest {
    pub payment_method: String,
    pub payment_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CustomizeOrderRequest {
    pub seat_selections: Option<Vec<SeatSelection>>,
    pub meal_selections: Option<Vec<MealSelection>>,
}

#[derive(Debug, Deserialize)]
pub struct SeatSelection {
    pub flight_id: String,
    pub passenger_index: u32,
    pub seat_number: String,
}

#[derive(Debug, Deserialize)]
pub struct MealSelection {
    pub flight_id: String,
    pub passenger_index: u32,
    pub meal_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FulfillmentResponse {
    pub order_id: Uuid,
    pub barcodes: Vec<BarcodeResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarcodeResponse {
    pub item_id: Uuid,
    pub barcode: String,
    pub qr_code_url: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /v1/orders/:id
/// Retrieve order details
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderResponse>, StatusCode> {
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// POST /v1/orders/:id/pay
/// Pay for an order
pub async fn pay_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<PayOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // 1. Get order to verify exists
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Process pay (Mock payment logic, but update DB status)
    state.order_repo.update_order_status(order_id, "PAID").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Generate fulfillment records (barcodes) for each item
    for item in &order.items {
        let barcode = format!("ALTIS-{}-{}", order_id.simple(), item.id.simple());
        let _ = state.order_repo.create_fulfillment(order_id, item.id, "BARCODE", &barcode).await;
    }

    // 4. Return updated order
    order.status = "PAID".to_string();
    Ok(Json(order))
}

/// POST /v1/orders/:id/customize
/// Customize order (select seats, meals)
pub async fn customize_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<CustomizeOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // Mock customization logic (metadata updates)
    // In production, this would update item metadata in order_items table
    
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// GET /v1/orders/:id/fulfillment
/// Get fulfillment details (barcodes, QR codes)
pub async fn get_fulfillment(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<FulfillmentResponse>, StatusCode> {
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Extraction: In the real repo, get_order returns fulfillment as a field
    let barcodes = if let Some(fulfillment) = order_json["fulfillment"].as_array() {
        fulfillment.iter().map(|f| BarcodeResponse {
            item_id: Uuid::parse_str(f["order_item_id"].as_str().unwrap_or_default()).unwrap_or_default(),
            barcode: f["barcode"].as_str().unwrap_or_default().to_string(),
            qr_code_url: Some(format!("https://api.altis.com/qr/{}", f["barcode"].as_str().unwrap_or_default())),
        }).collect()
    } else {
        vec![]
    };
    
    Ok(Json(FulfillmentResponse {
        order_id,
        barcodes,
    }))
}

/// POST /v1/orders/:id/cancel
/// Cancel an order
pub async fn cancel_order(
    State(_state): State<AppState>,
    Path(_order_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement order cancellation
    // 1. Verify order can be cancelled (not already fulfilled)
    // 2. Process refund if already paid
    // 3. Update order status to CANCELLED
    // 4. Release inventory
    // 5. Send cancellation email
    
    Ok(StatusCode::NO_CONTENT)
}

/// GET /v1/orders
/// List customer's orders
pub async fn list_orders(
    State(state): State<AppState>,
) -> Result<Json<Vec<OrderResponse>>, StatusCode> {
    // For now, list all orders since we don't have full JWT user context yet
    // In production, this would use customer_id from token
    let orders_json = state.order_repo.list_orders("").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let responses: Vec<OrderResponse> = orders_json.into_iter()
        .filter_map(|val| serde_json::from_value(val).ok())
        .collect();
    
    Ok(Json(responses))
}
