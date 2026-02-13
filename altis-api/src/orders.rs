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
    pub offer_id: Option<Uuid>,
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

#[derive(Debug, Deserialize)]
pub struct ConsumeFulfillmentRequest {
    pub location: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReshopOrderRequest {
    pub add_products: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ReshopOrderResponse {
    pub order_id: Uuid,
    pub new_total_nuc: i32,
    pub additional_nuc: i32,
    pub items_to_add: Vec<OrderItemResponse>,
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
    Json(_req): Json<PayOrderRequest>,
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

    // Log Audit Change
    let _ = state.order_repo.add_order_change(
        order_id, 
        "PAYMENT_RECEIVED", 
        Some(serde_json::json!({"status": "PROPOSED"})), 
        Some(serde_json::json!({"status": "PAID"})), 
        "SYSTEM", 
        Some("Order paid via API")
    ).await;

    // Log Telemetry
    let _ = state.telemetry.log_order_paid(altis_shared::models::events::OrderPaidEvent {
        order_id,
        offer_id: order.offer_id, // Need to add to OrderResponse or fetch
        customer_id: order.customer_id.clone(),
        total_nuc: order.total_nuc,
        timestamp: chrono::Utc::now().timestamp(),
    }).await;

    let _ = state.telemetry.log_settlement(altis_shared::models::events::SettlementEvent {
        order_id,
        amount_nuc: order.total_nuc,
        currency: order.currency.clone(),
        event_type: "PAYMENT".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    }).await;

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
    Json(_req): Json<CustomizeOrderRequest>,
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

/// POST /v1/fulfillment/:barcode/consume
/// Consume a barcode (Service Delivery)
pub async fn consume_fulfillment(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
    Json(req): Json<ConsumeFulfillmentRequest>,
) -> Result<StatusCode, StatusCode> {
    // 1. Get fulfillment to find order/amount
    // For simplicity in this phase, we skip the lookup and log basic settlement
    // In production, we'd fetch the order_item price
    
    state.order_repo.consume_fulfillment(&barcode, &req.location).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Log Settlement (Consumption)
    let _ = state.telemetry.log_settlement(altis_shared::models::events::SettlementEvent {
        order_id: Uuid::new_v4(), // Placeholder or from lookup
        amount_nuc: 0, 
        currency: "NUC".to_string(),
        event_type: "CONSUMPTION".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    }).await;
    
    Ok(StatusCode::OK)
}

/// POST /v1/orders/:id/reshop
/// Initial skeleton for post-booking modifications
pub async fn reshop_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<ReshopOrderRequest>,
) -> Result<Json<ReshopOrderResponse>, StatusCode> {
    // 1. Fetch current order
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Fetch products to add
    let mut items_to_add = Vec::new();
    let mut additional_nuc = 0;

    for product_id in req.add_products {
        let product = state.catalog_repo.get_product(product_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;

        let price = product["base_price_nuc"].as_i64().unwrap_or(0) as i32;
        additional_nuc += price;

        items_to_add.push(OrderItemResponse {
            id: Uuid::new_v4(),
            product_type: product["product_type"].as_str().unwrap_or("EXTRA").to_string(),
            name: product["name"].as_str().unwrap_or("Extra Product").to_string(),
            price_nuc: price,
            status: "CONFIRMED".to_string(),
            metadata: product["metadata"].clone(),
        });
    }

    // 3. Return proposal
    Ok(Json(ReshopOrderResponse {
        order_id,
        new_total_nuc: order.total_nuc + additional_nuc,
        additional_nuc,
        items_to_add,
    }))
}
