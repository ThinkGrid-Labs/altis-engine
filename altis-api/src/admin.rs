use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub product_type: String,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_nuc: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub airline_id: Uuid,
    pub product_type: String,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_nuc: i32,
    pub metadata: serde_json::Value,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePricingRuleRequest {
    pub rule_name: String,
    pub rule_type: String,
    pub product_id: Option<Uuid>,
    pub conditions: serde_json::Value,
    pub adjustments: serde_json::Value,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingRuleResponse {
    pub id: Uuid,
    pub airline_id: Uuid,
    pub rule_name: String,
    pub rule_type: String,
    pub conditions: serde_json::Value,
    pub adjustments: serde_json::Value,
    pub priority: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateBundleRequest {
    pub bundle_name: String,
    pub bundle_type: String,
    pub product_types: serde_json::Value,
    pub discount_percentage: Option<f64>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleResponse {
    pub id: Uuid,
    pub airline_id: Uuid,
    pub bundle_name: String,
    pub bundle_type: String,
    pub product_types: serde_json::Value,
    pub discount_percentage: f64,
    pub priority: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub product_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerDisruptionRequest {
    pub flight_id: Uuid,
    pub new_status: String, // DELAYED, CANCELLED
}

// ============================================================================
// Product Management Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/products
pub async fn create_product(
    State(state): State<AppState>,
    Path(airline_id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let product_json = serde_json::json!({
        "airline_id": airline_id,
        "product_type": req.product_type,
        "product_code": req.product_code,
        "name": req.name,
        "description": req.description,
        "base_price_nuc": req.base_price_nuc,
        "metadata": req.metadata.clone().unwrap_or(serde_json::json!({})),
    });

    let product_id = state.catalog_repo.create_product(&product_json).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ProductResponse {
        id: product_id,
        airline_id,
        product_type: req.product_type,
        product_code: req.product_code,
        name: req.name,
        description: req.description,
        base_price_nuc: req.base_price_nuc,
        metadata: req.metadata.unwrap_or(serde_json::json!({})),
        is_active: true,
    }))
}

/// GET /v1/admin/airlines/:airline_id/products
pub async fn list_products(
    State(state): State<AppState>,
    Path(airline_id): Path<Uuid>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Vec<ProductResponse>>, StatusCode> {
    let products_json = state.catalog_repo.list_products(airline_id, query.product_type.as_deref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let responses: Vec<ProductResponse> = products_json.into_iter()
        .filter_map(|val| serde_json::from_value(val).ok())
        .collect();
    
    Ok(Json(responses))
}

/// GET /v1/admin/products/:id
pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let product_json = state.catalog_repo.get_product(product_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response: ProductResponse = serde_json::from_value(product_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// PUT /v1/admin/products/:id
pub async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    let product_json = serde_json::json!({
        "product_type": req.product_type,
        "product_code": req.product_code,
        "name": req.name,
        "description": req.description,
        "base_price_nuc": req.base_price_nuc,
        "metadata": req.metadata.unwrap_or(serde_json::json!({})),
    });

    state.catalog_repo.update_product(product_id, &product_json).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let updated = state.catalog_repo.get_product(product_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response: ProductResponse = serde_json::from_value(updated)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// DELETE /v1/admin/products/:id
pub async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.catalog_repo.delete_product(product_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Pricing Rules Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/pricing-rules
pub async fn create_pricing_rule(
    State(_state): State<AppState>,
    Path(airline_id): Path<Uuid>,
    Json(req): Json<CreatePricingRuleRequest>,
) -> Result<Json<PricingRuleResponse>, StatusCode> {
    // Create pricing rule
    let rule_id = Uuid::new_v4();
    
    Ok(Json(PricingRuleResponse {
        id: rule_id,
        airline_id,
        rule_name: req.rule_name,
        rule_type: req.rule_type,
        conditions: req.conditions,
        adjustments: req.adjustments,
        priority: req.priority.unwrap_or(10),
        is_active: true,
    }))
}

/// GET /v1/admin/airlines/:airline_id/pricing-rules
pub async fn list_pricing_rules(
    State(_state): State<AppState>,
    Path(airline_id): Path<Uuid>,
) -> Result<Json<Vec<PricingRuleResponse>>, StatusCode> {
    // Mock pricing rules
    let rules = vec![
        PricingRuleResponse {
            id: Uuid::new_v4(),
            airline_id,
            rule_name: "Continuous Pricing - Economy".to_string(),
            rule_type: "DEMAND".to_string(),
            conditions: serde_json::json!({"cabin_class": "ECONOMY"}),
            adjustments: serde_json::json!({
                "type": "FORMULA",
                "formula": "1.0 + (utilization^2 * 2.0)"
            }),
            priority: 10,
            is_active: true,
        },
    ];
    
    Ok(Json(rules))
}

/// GET /v1/admin/pricing-rules/:id
pub async fn get_pricing_rule(
    State(_state): State<AppState>,
    Path(_rule_id): Path<Uuid>,
) -> Result<Json<PricingRuleResponse>, StatusCode> {
    // TODO: Implement pricing rule retrieval
    Err(StatusCode::NOT_FOUND)
}

/// PUT /v1/admin/pricing-rules/:id
pub async fn update_pricing_rule(
    State(_state): State<AppState>,
    Path(_rule_id): Path<Uuid>,
    Json(_req): Json<CreatePricingRuleRequest>,
) -> Result<Json<PricingRuleResponse>, StatusCode> {
    // TODO: Implement pricing rule update
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// DELETE /v1/admin/pricing-rules/:id
pub async fn delete_pricing_rule(
    State(_state): State<AppState>,
    Path(_rule_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement pricing rule deletion
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Bundle Templates Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/bundles
pub async fn create_bundle(
    State(_state): State<AppState>,
    Path(airline_id): Path<Uuid>,
    Json(req): Json<CreateBundleRequest>,
) -> Result<Json<BundleResponse>, StatusCode> {
    // Create bundle
    let bundle_id = Uuid::new_v4();
    
    Ok(Json(BundleResponse {
        id: bundle_id,
        airline_id,
        bundle_name: req.bundle_name,
        bundle_type: req.bundle_type,
        product_types: req.product_types,
        discount_percentage: req.discount_percentage.unwrap_or(0.0),
        priority: req.priority.unwrap_or(1),
        is_active: true,
    }))
}

/// GET /v1/admin/airlines/:airline_id/bundles
pub async fn list_bundles(
    State(_state): State<AppState>,
    Path(airline_id): Path<Uuid>,
) -> Result<Json<Vec<BundleResponse>>, StatusCode> {
    // Mock bundles
    let bundles = vec![
        BundleResponse {
            id: Uuid::new_v4(),
            airline_id,
            bundle_name: "Comfort Bundle".to_string(),
            bundle_type: "COMFORT".to_string(),
            product_types: serde_json::json!([
                {"type": "FLIGHT", "required": true},
                {"type": "SEAT", "category": "EXTRA_LEGROOM"},
                {"type": "MEAL", "category": "HOT"}
            ]),
            discount_percentage: 10.0,
            priority: 2,
            is_active: true,
        },
    ];
    
    Ok(Json(bundles))
}

/// GET /v1/admin/bundles/:id
pub async fn get_bundle(
    State(_state): State<AppState>,
    Path(_bundle_id): Path<Uuid>,
) -> Result<Json<BundleResponse>, StatusCode> {
    // TODO: Implement bundle retrieval
    Err(StatusCode::NOT_FOUND)
}

/// PUT /v1/admin/bundles/:id
pub async fn update_bundle(
    State(_state): State<AppState>,
    Path(_bundle_id): Path<Uuid>,
    Json(_req): Json<CreateBundleRequest>,
) -> Result<Json<BundleResponse>, StatusCode> {
    // TODO: Implement bundle update
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// DELETE /v1/admin/bundles/:id
pub async fn delete_bundle(
    State(_state): State<AppState>,
    Path(_bundle_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement bundle deletion
    Ok(StatusCode::NO_CONTENT)
}
pub async fn trigger_disruption(
    State(state): State<AppState>,
    Json(req): Json<TriggerDisruptionRequest>,
) -> Result<StatusCode, StatusCode> {
    // 1. Fetch flight details to know origin/destination
    let flight_json = state.catalog_repo.get_product(req.flight_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let origin = flight_json["metadata"]["origin"].as_str().unwrap_or_default();
    let destination = flight_json["metadata"]["destination"].as_str().unwrap_or_default();
    let airline_id = Uuid::parse_str(flight_json["airline_id"].as_str().unwrap_or_default()).unwrap_or_default();

    // 2. Find all affected orders
    let affected_orders = state.order_repo.find_orders_by_flight(&req.flight_id.to_string()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Found {} affected orders for flight {} ({}-{})", affected_orders.len(), req.flight_id, origin, destination);

    // 3. Search for alternative flight (same route, different ID)
    let alt_flights = state.catalog_repo.list_products(airline_id, Some("FLIGHT")).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let alternative = alt_flights.iter()
        .find(|f| {
            f["metadata"]["origin"] == origin && 
            f["metadata"]["destination"] == destination && 
            f["id"] != req.flight_id.to_string()
        });

    // 4. Update orders
    for order_val in affected_orders {
        let order_id = Uuid::parse_str(order_val["id"].as_str().unwrap_or_default()).unwrap_or_default();
        
        // Log Audit Change
        let _ = state.order_repo.add_order_change(
            order_id,
            "FLIGHT_DISRUPTION",
            None,
            Some(serde_json::json!({"flight_id": req.flight_id, "new_status": req.new_status})),
            "ADMIN",
            Some("Flight disruption triggered by admin")
        ).await;

        // Add Re-accommodation if alternative found
        if let Some(alt) = alternative {
            let mut metadata = alt["metadata"].clone();
            metadata["disrupted_flight_id"] = serde_json::json!(req.flight_id.to_string());
            
            let reac_item = serde_json::json!({
                "product_type": "FLIGHT",
                "product_id": alt["id"],
                "name": alt["name"],
                "price_nuc": 0, // Involuntary re-accommodation is free
                "status": "REACCOMMODATED",
                "metadata": metadata
            });

            let _ = state.order_repo.add_order_item(order_id, &reac_item).await;
        }
    }

    Ok(StatusCode::OK)
}
