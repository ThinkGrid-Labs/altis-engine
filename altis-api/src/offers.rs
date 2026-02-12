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
    let domain_context = altis_offer::features::SearchContext {
        origin: req.origin.clone(),
        destination: req.destination.clone(),
        departure_date: req.departure_date.clone(),
        passengers: req.passengers as i32,
        cabin_class: req.cabin_class.clone(),
    };

    let search_context_json = serde_json::to_value(&domain_context).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 2. Fetch products from catalog
    // For now, we fetch AA products as a default if no specific airline is in context
    let airline_id = Uuid::nil(); // TODO: Determine airline from route/config
    let products = state.catalog_repo.list_products(airline_id, None).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Helper to find product by code
    let find_product = |code: &str| {
        products.iter().find(|p| p["product_code"].as_str() == Some(code))
    };

    // 3. Generate offers (Flight-Only, Comfort, Premium)
    let mut offers = Vec::new();
    
    // Flight-Only offer
    let mut flight_only = altis_offer::Offer::new(None, Some(airline_id), search_context_json.clone());
    flight_only.add_item(altis_offer::OfferItem::new(
        "FLIGHT".to_string(),
        None,
        Some("FLIGHT-BASE".to_string()),
        format!("{} to {}", req.origin, req.destination),
        Some(format!("Direct flight on {}", req.departure_date)),
        20000, 
        req.passengers as i32,
        serde_json::json!({"origin": req.origin, "destination": req.destination}),
    ));
    offers.push(flight_only);
    
    // Comfort Bundle
    let mut comfort = altis_offer::Offer::new(None, Some(airline_id), search_context_json.clone());
    comfort.add_item(altis_offer::OfferItem::new(
        "FLIGHT".to_string(),
        None,
        Some("FLIGHT-BASE".to_string()),
        format!("{} to {}", req.origin, req.destination),
        None,
        20000,
        req.passengers as i32,
        serde_json::json!({}),
    ));
    
    if let Some(seat) = find_product("SEAT-EXTRA-LEG") {
        comfort.add_item(altis_offer::OfferItem::new(
            "SEAT".to_string(),
            None,
            Some("SEAT-EXTRA-LEG".to_string()),
            seat["name"].as_str().unwrap_or("Seat").to_string(),
            seat["description"].as_str().map(|s| s.to_string()),
            (seat["base_price_nuc"].as_i64().unwrap_or(3000) as f64 * 0.9) as i32,
            req.passengers as i32,
            seat["metadata"].clone(),
        ));
    }

    if let Some(meal) = find_product("MEAL-HOT") {
        comfort.add_item(altis_offer::OfferItem::new(
            "MEAL".to_string(),
            None,
            Some("MEAL-HOT".to_string()),
            meal["name"].as_str().unwrap_or("Meal").to_string(),
            meal["description"].as_str().map(|s| s.to_string()),
            (meal["base_price_nuc"].as_i64().unwrap_or(1500) as f64 * 0.9) as i32,
            req.passengers as i32,
            meal["metadata"].clone(),
        ));
    }
    offers.push(comfort);
    
    // 4. AI Ranking
    let mut ranker = state.ranker.lock().await;
    ranker.rank_offers_with_context(&domain_context, &mut offers).await;
    
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
    Path(offer_id): Path<Uuid>,
    Json(req): Json<AcceptOfferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 1. Get offer to verify and log
    let offer_json = state.offer_repo.get_offer(offer_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let offer: altis_offer::Offer = serde_json::from_value(offer_json.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Log Telemetry
    let _ = state.telemetry.log_offer_accepted(altis_shared::models::events::OfferAcceptedEvent {
        offer_id,
        customer_id: Some(req.customer_email.clone()),
        timestamp: chrono::Utc::now().timestamp(),
    }).await;

    // 3. Create Order
    let order_id = state.order_repo.create_order(&serde_json::json!({
        "customer_id": req.customer_email,
        "customer_email": req.customer_email,
        "offer_id": offer_id,
        "status": "PROPOSED",
        "total_nuc": offer.total_nuc,
        "currency": offer.currency,
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
