use axum::{
    extract::{Path, Query, State},
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

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub product_type: String,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_nuc: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

// ============================================================================
// Product Management Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/products
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    Path(airline_id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    // Create product
    let product_id = Uuid::new_v4();
    
    // Return product response
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
    State(state): State<Arc<AppState>>,
    Path(airline_id): Path<Uuid>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Vec<ProductResponse>>, StatusCode> {
    // Mock product list
    let products = vec![
        ProductResponse {
            id: Uuid::new_v4(),
            airline_id,
            product_type: "SEAT".to_string(),
            product_code: "SEAT-EXTRA-LEG".to_string(),
            name: "Extra Legroom Seat".to_string(),
            description: Some("34-36 inches of legroom".to_string()),
            base_price_nuc: 3000,
            metadata: serde_json::json!({"category": "EXTRA_LEGROOM"}),
            is_active: true,
        },
        ProductResponse {
            id: Uuid::new_v4(),
            airline_id,
            product_type: "MEAL".to_string(),
            product_code: "MEAL-HOT".to_string(),
            name: "Hot Meal".to_string(),
            description: Some("Chef-prepared hot meal".to_string()),
            base_price_nuc: 1500,
            metadata: serde_json::json!({"category": "HOT"}),
            is_active: true,
        },
    ];
    
    // Filter by product_type if specified
    let filtered = if let Some(product_type) = &query.product_type {
        products.into_iter()
            .filter(|p| &p.product_type == product_type)
            .collect()
    } else {
        products
    };
    
    Ok(Json(filtered))
}

/// GET /v1/admin/products/:id
pub async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ProductResponse>, StatusCode> {
    // TODO: Implement product retrieval
    Err(StatusCode::NOT_FOUND)
}

/// PUT /v1/admin/products/:id
pub async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    // TODO: Implement product update
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// DELETE /v1/admin/products/:id
pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement product deletion (soft delete)
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Pricing Rules Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/pricing-rules
pub async fn create_pricing_rule(
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<PricingRuleResponse>, StatusCode> {
    // TODO: Implement pricing rule retrieval
    Err(StatusCode::NOT_FOUND)
}

/// PUT /v1/admin/pricing-rules/:id
pub async fn update_pricing_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
    Json(req): Json<CreatePricingRuleRequest>,
) -> Result<Json<PricingRuleResponse>, StatusCode> {
    // TODO: Implement pricing rule update
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// DELETE /v1/admin/pricing-rules/:id
pub async fn delete_pricing_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement pricing rule deletion
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Bundle Templates Handlers
// ============================================================================

/// POST /v1/admin/airlines/:airline_id/bundles
pub async fn create_bundle(
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
    Path(bundle_id): Path<Uuid>,
) -> Result<Json<BundleResponse>, StatusCode> {
    // TODO: Implement bundle retrieval
    Err(StatusCode::NOT_FOUND)
}

/// PUT /v1/admin/bundles/:id
pub async fn update_bundle(
    State(state): State<Arc<AppState>>,
    Path(bundle_id): Path<Uuid>,
    Json(req): Json<CreateBundleRequest>,
) -> Result<Json<BundleResponse>, StatusCode> {
    // TODO: Implement bundle update
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// DELETE /v1/admin/bundles/:id
pub async fn delete_bundle(
    State(state): State<Arc<AppState>>,
    Path(bundle_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement bundle deletion
    Ok(StatusCode::NO_CONTENT)
}
