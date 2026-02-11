use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct FulfillmentResponse {
    pub order_id: Uuid,
    pub barcodes: Vec<BarcodeResponse>,
}

#[derive(Debug, Serialize)]
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
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // TODO: Implement order retrieval
    // 1. Fetch order from database
    // 2. Verify customer owns this order (from JWT)
    // 3. Return order details
    
    Err(StatusCode::NOT_FOUND)
}

/// POST /v1/orders/:id/pay
/// Pay for an order
pub async fn pay_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<PayOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // TODO: Implement payment processing
    // 1. Verify order is in PROPOSED status
    // 2. Process payment (integrate with payment gateway)
    // 3. Update order status to PAID
    // 4. Permanently lock inventory (seats, meals)
    // 5. Trigger fulfillment (generate barcodes)
    // 6. Send confirmation email
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /v1/orders/:id/customize
/// Customize order (select seats, meals)
pub async fn customize_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<CustomizeOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // TODO: Implement order customization
    // 1. Verify order is in PROPOSED status
    // 2. Hold selected seats in Redis (15-min TTL)
    // 3. Update order item metadata with selections
    // 4. Return updated order
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// GET /v1/orders/:id/fulfillment
/// Get fulfillment details (barcodes, QR codes)
pub async fn get_fulfillment(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<FulfillmentResponse>, StatusCode> {
    // TODO: Implement fulfillment retrieval
    // 1. Verify order is PAID or FULFILLED
    // 2. Fetch barcodes from fulfillment table
    // 3. Generate QR code URLs
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /v1/orders/:id/cancel
/// Cancel an order
pub async fn cancel_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<OrderResponse>>, StatusCode> {
    // TODO: Implement order listing
    // 1. Get customer_id from JWT
    // 2. Fetch all orders for customer
    // 3. Return paginated list
    
    Ok(Json(vec![]))
}
