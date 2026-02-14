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

#[derive(Debug, Serialize)]
pub struct PaymentIntentResponse {
    pub intent_id: String,
    pub amount: i32,
    pub currency: String,
    pub client_secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub offer_id: Option<Uuid>,
    pub customer_id: String,
    pub customer_email: Option<altis_shared::pii::Masked<String>>,
    pub customer_did: Option<String>,
    pub status: String,
    pub items: Vec<OrderItemResponse>,
    pub travelers: Option<Vec<altis_core::iata::Traveler>>,
    pub contact_info: Option<altis_core::iata::ContactInfo>,
    pub total_nuc: i32,
    pub currency: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub product_type: String,
    pub name: String,
    pub price_nuc: i32,
    pub status: String,
    pub revenue_status: String,
    pub operating_carrier_id: Option<Uuid>,
    pub net_rate_nuc: Option<i32>,
    pub commission_nuc: Option<i32>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PayOrderRequest {
    pub payment_method: String,
    pub payment_token: String,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptReaccommodationRequest {
    pub selected_item_ids: Vec<Uuid>,
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
    Json(req): Json<PayOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // 1. Get order to verify exists
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 1.5 Verify order is not expired
    if let Some(expires_at) = order.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err(StatusCode::GONE);
        }
    }

    // 1.5 Verify order is not expired
    if let Some(expires_at) = order.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err(StatusCode::GONE);
        }
    }

    // 2. Lock-in: Transition to PAYMENT_PENDING
    // This prevents the background cleanup worker from releasing inventory
    state.order_repo.update_order_status(order_id, "PAYMENT_PENDING").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Process pay via Orchestrator
    let intent = altis_core::payment::PaymentIntent {
        id: format!("pi_{}", order_id.simple()),
        order_id,
        amount: order.total_nuc,
        currency: order.currency.clone(),
        status: altis_core::payment::PaymentStatus::RequiresPaymentMethod,
        reference: req.payment_reference.clone(),
        client_secret: None,
        created_at: chrono::Utc::now(),
    };

    let payment_status = state.payment_orchestrator.process_payment(&intent).await
        .map_err(|e| {
            tracing::error!("Payment Orchestration Failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR // This will be caught by CB middleware
        })?;

    if payment_status != altis_core::payment::PaymentStatus::Succeeded {
        // If it's still processing (async), we stay in PAYMENT_PENDING
        if payment_status == altis_core::payment::PaymentStatus::Processing {
             return Ok(Json(order));
        }
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

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

/// POST /v1/orders/:id/payment-intent
/// Initialize a payment intent for the order
pub async fn initialize_payment_intent(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<PaymentIntentResponse>, StatusCode> {
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 1.5 Verify order is not expired
    if let Some(expires_at) = order.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err(StatusCode::GONE);
        }
    }

    let intent = state.payment_orchestrator.initialize_payment(
        order_id, 
        order.total_nuc, 
        &order.currency
    ).await.map_err(|e| {
        tracing::error!("Failed to initialize payment: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(PaymentIntentResponse {
        intent_id: intent.id,
        amount: intent.amount,
        currency: intent.currency,
        client_secret: intent.client_secret,
    }))
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
            qr_code_url: Some(format!("{}/qr/{}", state.api_base_url, f["barcode"].as_str().unwrap_or_default())),
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
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // 1. Get order to verify exists and check status
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if order.status == "CANCELLED" {
        return Ok(StatusCode::NO_CONTENT);
    }

    // 2. Update order status to CANCELLED
    state.order_repo.update_order_status(order_id, "CANCELLED").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Release inventory
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

    // 4. Log Audit Change
    let _ = state.order_repo.add_order_change(
        order_id, 
        "CANCELLED", 
        Some(serde_json::json!({"status": order.status})), 
        Some(serde_json::json!({"status": "CANCELLED"})), 
        "CUSTOMER", 
        Some("Order cancelled via API")
    ).await;
    
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
    // 1. Consume fulfillment and get IDs
    let (order_id, item_id) = state.order_repo.consume_fulfillment(&barcode, &req.location).await
        .map_err(|e| {
            tracing::error!("Failed to consume fulfillment for barcode {}: {:?}", barcode, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 2. Fetch order to get price and current status
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch order {} after consumption: {:?}", order_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::error!("Order {} not found after consumption", order_id);
            StatusCode::NOT_FOUND
        })?;

    let order: altis_order::Order = serde_json::from_value(order_json.clone())
        .map_err(|e| {
            tracing::error!("Failed to deserialize order {} after consumption. Full JSON: {:?}. Error: {:?}", order_id, order_json, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 3. Recognize Revenue
    let financial_mgr = altis_order::finance::FinancialManager::new();
    if let Some(entry) = financial_mgr.recognize_revenue(&order, item_id) {
        // Save Ledger Entry
        let _ = state.order_repo.add_order_ledger_entry(
            entry.order_id,
            entry.order_item_id,
            &entry.transaction_type,
            entry.amount_nuc,
            entry.description.as_deref(),
        ).await;

        // Update Revenue Status to EARNED
        let _ = state.order_repo.update_item_revenue_status(item_id, "EARNED").await;

        // 4. Log Settlement (Consumption)
        let _ = state.telemetry.log_settlement(altis_shared::models::events::SettlementEvent {
            order_id,
            amount_nuc: entry.amount_nuc,
            currency: order.currency.clone(),
            event_type: "REVENUE_RECOGNITION".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }).await;
    }
    
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
            product_id: Some(product_id),
            product_type: product["product_type"].as_str().unwrap_or("EXTRA").to_string(),
            name: product["name"].as_str().unwrap_or("Extra Product").to_string(),
            price_nuc: price,
            status: "CONFIRMED".to_string(),
            revenue_status: "UNEARNED".to_string(),
            operating_carrier_id: None,
            net_rate_nuc: None,
            commission_nuc: None,
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

/// POST /v1/orders/:id/accept-reaccommodation
/// Accept proposed re-accommodation items
pub async fn accept_reaccommodation(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<AcceptReaccommodationRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // 1. Fetch current order
    let _order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 2. Process acceptance (Mock logic)
    // In a real repo, we'd update specific item statuses
    let _ = state.order_repo.add_order_change(
        order_id,
        "REACCOMMODATION_ACCEPTED",
        None,
        Some(serde_json::json!({"accepted_items": req.selected_item_ids})),
        "CUSTOMER",
        Some("Customer accepted alternative flight")
    ).await;

    // 3. Return updated order
    let updated_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response: OrderResponse = serde_json::from_value(updated_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// POST /v1/orders/:id/involuntary-refund
/// Process a full refund for a disrupted flight (zero fees)
pub async fn involuntary_refund(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // 1. Update order status to CANCELLED
    state.order_repo.update_order_status(order_id, "CANCELLED").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Log Audit Change
    let _ = state.order_repo.add_order_change(
        order_id,
        "INVOLUNTARY_REFUND",
        None,
        None,
        "SYSTEM",
        Some("Full refund processed due to flight disruption")
    ).await;

    Ok(StatusCode::OK)
}
