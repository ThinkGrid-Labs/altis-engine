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

#[derive(Debug, Deserialize)]
pub struct SearchOffersRequest {
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
    pub return_date: Option<String>,
    pub passengers: u32,
    pub cabin_class: Option<String>,
    pub user_segment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptReaccommodationRequest {
    pub selected_item_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ReshopOrderResponse {
    pub id: Uuid,
    pub items: Vec<OfferItemResponse>,
    pub total_nuc: i32,
    pub currency: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct OfferResponse {
    pub id: Uuid,
    pub items: Vec<OfferItemResponse>,
    pub total_nuc: i32,
    pub currency: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct OfferItemResponse {
    pub id: Uuid,
    pub product_type: String,
    pub name: String,
    pub description: Option<String>,
    pub price_nuc: i32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AcceptOfferRequest {
    pub customer_email: String,
    pub travelers: Option<Vec<altis_core::iata::Traveler>>,
    pub contact_info: Option<altis_core::iata::ContactInfo>,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /v1/offers/search
/// Generate offers based on search criteria
pub async fn search_offers(
    State(state): State<AppState>,
    Json(req): Json<SearchOffersRequest>,
) -> Result<Json<Vec<OfferResponse>>, StatusCode> {
    // 1. Build search context
    let search_context = altis_offer::features::SearchContext {
        origin: req.origin.clone(),
        destination: req.destination.clone(),
        departure_date: req.departure_date.clone(),
        passengers: req.passengers as i32, // Assuming SearchContext still expects i32
        cabin_class: None, // TODO: Pull from request if available
        user_segment: req.user_segment.clone(),
    };

    let search_context_json = serde_json::to_value(&search_context).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 2. Fetch products from catalog
    // Dynamically find AirAltis LCC (AL) ID
    let airline = state.catalog_repo.get_airline_by_code("AL").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?; // AL must exist from migration
    
    let airline_id = Uuid::parse_str(airline["id"].as_str().unwrap_or_default()).unwrap_or_default();
    
    let products = state.catalog_repo.list_products(airline_id, None).await
        .map_err(|e| {
            tracing::error!("Failed to fetch products for airline {}: {:?}", airline_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Helper to find product by code
    let _find_product = |code: &str| {
        products.iter().find(|p| p["product_code"].as_str() == Some(code))
    };

    // 3. Generate offers using dynamic OfferGenerator
    let generator = altis_offer::generator::OfferGenerator::new(
        altis_catalog::pricing::PricingEngine::new(altis_catalog::pricing::PricingConfig::default())
    );

    // Convert catalog products to domain Products
    let domain_products: Vec<altis_catalog::Product> = products.into_iter().map(|p| {
        altis_catalog::Product {
            id: Uuid::parse_str(p["id"].as_str().unwrap_or_default()).unwrap_or_default(),
            product_type: serde_json::from_value(p["product_type"].clone()).unwrap_or(altis_catalog::ProductType::Flight),
            product_code: p["product_code"].as_str().unwrap_or_default().to_string(),
            name: p["name"].as_str().unwrap_or_default().to_string(),
            description: p["description"].as_str().map(|s| s.to_string()),
            base_price_nuc: p["base_price_nuc"].as_i64().unwrap_or(0) as i32,
            margin_percentage: p["margin_percentage"].as_f64().unwrap_or(0.15),
            is_active: p["is_active"].as_bool().unwrap_or(true),
            metadata: p["metadata"].clone(),
        }
    }).collect();

    let (flights, ancillaries): (Vec<_>, Vec<_>) = domain_products.into_iter()
        .partition(|p| p.product_type == altis_catalog::ProductType::Flight);

    let mut offers = generator.generate_offers(
        None, // customer_id
        req.user_segment.clone(),
        search_context_json.clone(),
        flights,
        ancillaries,
    ).await.map_err(|e| {
        tracing::error!("Offer generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // 4. AI Ranking
    let mut ranker = state.ranker.lock().await;
    ranker.rank_offers_with_context(&search_context, &mut offers).await;
    
    // 5. Save generated offers to repository (for retrieval on accept)
    for offer in &offers {
        let val = serde_json::to_value(offer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state.offer_repo.save_offer(&val).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 6. Convert to response format
    let responses: Vec<OfferResponse> = offers.into_iter()
        .map(|offer| OfferResponse {
            id: offer.id,
            items: offer.items.iter().map(|item| OfferItemResponse {
                id: item.id,
                product_type: item.product_type.clone(),
                name: item.name.clone(),
                description: item.description.clone(),
                price_nuc: item.price_nuc,
                metadata: item.metadata.clone(),
            }).collect(),
            total_nuc: offer.total_nuc,
            currency: offer.currency.clone(),
            expires_at: offer.expires_at,
        })
        .collect();
    
    Ok(Json(responses))
}

/// GET /v1/offers/:id
/// Retrieve a specific offer
pub async fn get_offer(
    State(state): State<AppState>,
    Path(offer_id): Path<Uuid>,
) -> Result<Json<OfferResponse>, StatusCode> {
    let offer_json = state.offer_repo.get_offer(offer_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let offer: altis_offer::Offer = serde_json::from_value(offer_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if offer.is_expired() {
        return Err(StatusCode::GONE);
    }

    let response = OfferResponse {
        id: offer.id,
        items: offer.items.iter().map(|item| OfferItemResponse {
            id: item.id,
            product_type: item.product_type.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            price_nuc: item.price_nuc,
            metadata: item.metadata.clone(),
        }).collect(),
        total_nuc: offer.total_nuc,
        currency: offer.currency.clone(),
        expires_at: offer.expires_at,
    };
    
    Ok(Json(response))
}

/// POST /v1/offers/:id/accept
/// Accept an offer and create an order
pub async fn accept_offer(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::middleware::auth::CustomerClaims>,
    Path(offer_id): Path<Uuid>,
    Json(req): Json<AcceptOfferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 1. Get offer to verify and log
    let offer_json = state.offer_repo.get_offer(offer_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let offer: altis_offer::Offer = serde_json::from_value(offer_json.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 1.5 Verify offer is not expired
    if offer.is_expired() {
        return Err(StatusCode::GONE);
    }

    // 2. Log Telemetry
    let _ = state.telemetry.log_offer_accepted(altis_shared::models::events::OfferAcceptedEvent {
        offer_id,
        customer_id: Some(req.customer_email.clone()),
        timestamp: chrono::Utc::now().timestamp(),
    }).await;

    // 3. Create Order
    // If sub starts with did:, use it as customer_did
    let (customer_id, customer_did) = if claims.sub.starts_with("did:") {
        (format!("DID-{}", &claims.sub.chars().take(12).collect::<String>()), Some(claims.sub.clone()))
    } else {
        (claims.sub.clone(), None)
    };

    // Calculate expiration based on airline rules or global default
    let airline_id = offer.airline_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?; 
    let hold_seconds = if let Ok(Some(rule)) = state.catalog_repo.get_inventory_rule(airline_id, "FLIGHT").await {
        rule["hold_duration_seconds"].as_u64().unwrap_or(state.business_rules.trip_hold_seconds)
    } else {
        state.business_rules.trip_hold_seconds
    };

    let expires_at = (chrono::Utc::now() + chrono::Duration::seconds(hold_seconds as i64)).to_rfc3339();

    // 4. Reserve Inventory (Hard Hold)
    for item in &offer.items {
        if item.product_type == "Flight" {
            if let Some(product_id) = item.product_id {
                let pid_str = product_id.to_string();
                match state.redis.decr_flight_availability(&pid_str).await {
                    Ok(Some(remaining)) if remaining < 0 => {
                        // Rollback: increment back (simple version for now)
                        let _ = state.redis.set_flight_availability(&pid_str, 0).await;
                        return Err(StatusCode::CONFLICT); // Seat just taken
                    }
                    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                    _ => {}
                }
            }
        }
    }

    let order_id = state.order_repo.create_order(&serde_json::json!({
        "customer_id": customer_id,
        "customer_email": req.customer_email,
        "customer_did": customer_did,
        "offer_id": offer_id,
        "status": "PROPOSED",
        "total_nuc": offer.total_nuc,
        "currency": offer.currency,
        "contact_phone": req.contact_info.as_ref().and_then(|c| c.phone.clone()),
        "contact_first_name": req.contact_info.as_ref().and_then(|c| c.first_name.clone()),
        "contact_last_name": req.contact_info.as_ref().and_then(|c| c.last_name.clone()),
        "travelers": req.travelers,
        "expires_at": expires_at,
    })).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Add Order Items
    for item in &offer.items {
        let _ = state.order_repo.add_order_item(order_id, &serde_json::to_value(item).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?).await;
    }
    
    // 5. Release/Update offer status (optional, usually done by expiry or order link)
    
    Ok(Json(serde_json::json!({
        "order_id": order_id,
        "status": "PROPOSED",
        "message": "Order created successfully. Proceed to payment.",
        "customer_email": req.customer_email,
    })))
}

/// DELETE /v1/offers/:id
/// Expire an offer (customer cancels)
pub async fn expire_offer(
    State(_state): State<AppState>,
    Path(_offer_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement offer expiry
    // 1. Mark offer as EXPIRED in database
    // 2. Remove from Redis cache
    // 3. Release any held inventory
    
    Ok(StatusCode::NO_CONTENT)
}
