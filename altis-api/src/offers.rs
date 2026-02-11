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
    let search_context = serde_json::json!({
        "origin": req.origin,
        "destination": req.destination,
        "departure_date": req.departure_date,
        "return_date": req.return_date,
        "passengers": req.passengers,
        "cabin_class": req.cabin_class,
    });
    
    // 2. Generate offers (Flight-Only, Comfort, Premium)
    let mut offers = Vec::new();
    
    // Flight-Only offer (baseline)
    let mut flight_only = altis_offer::Offer::new(None, None, search_context.clone());
    flight_only.add_item(altis_offer::OfferItem::new(
        "FLIGHT".to_string(),
        None,
        Some("FLIGHT-BASE".to_string()),
        format!("{} to {}", req.origin, req.destination),
        Some(format!("Direct flight on {}", req.departure_date)),
        20000, // Base price: 200 NUC
        req.passengers as i32,
        serde_json::json!({
            "origin": req.origin,
            "destination": req.destination,
            "date": req.departure_date,
        }),
    ));
    offers.push(flight_only);
    
    // Comfort Bundle (Flight + Seat + Meal with 10% discount)
    let mut comfort = altis_offer::Offer::new(None, None, search_context.clone());
    comfort.add_item(altis_offer::OfferItem::new(
        "FLIGHT".to_string(),
        None,
        Some("FLIGHT-BASE".to_string()),
        format!("{} to {}", req.origin, req.destination),
        Some(format!("Direct flight on {}", req.departure_date)),
        20000,
        req.passengers as i32,
        serde_json::json!({"origin": req.origin, "destination": req.destination}),
    ));
    comfort.add_item(altis_offer::OfferItem::new(
        "SEAT".to_string(),
        None,
        Some("SEAT-EXTRA-LEG".to_string()),
        "Extra Legroom Seat".to_string(),
        Some("34-36 inches of legroom".to_string()),
        2700, // 3000 NUC with 10% discount
        req.passengers as i32,
        serde_json::json!({"category": "EXTRA_LEGROOM"}),
    ));
    comfort.add_item(altis_offer::OfferItem::new(
        "MEAL".to_string(),
        None,
        Some("MEAL-HOT".to_string()),
        "Hot Meal".to_string(),
        Some("Chef-prepared hot meal".to_string()),
        1350, // 1500 NUC with 10% discount
        req.passengers as i32,
        serde_json::json!({"category": "HOT"}),
    ));
    offers.push(comfort);
    
    // Premium Bundle (Flight + Lounge + Fast Track)
    let mut premium = altis_offer::Offer::new(None, None, search_context.clone());
    premium.add_item(altis_offer::OfferItem::new(
        "FLIGHT".to_string(),
        None,
        Some("FLIGHT-BASE".to_string()),
        format!("{} to {}", req.origin, req.destination),
        Some(format!("Direct flight on {}", req.departure_date)),
        20000,
        req.passengers as i32,
        serde_json::json!({"origin": req.origin, "destination": req.destination}),
    ));
    premium.add_item(altis_offer::OfferItem::new(
        "LOUNGE".to_string(),
        None,
        Some("LOUNGE-ACCESS".to_string()),
        "Airport Lounge Access".to_string(),
        Some("Premium lounge with food and drinks".to_string()),
        4250, // 5000 NUC with 15% discount
        req.passengers as i32,
        serde_json::json!({"category": "PREMIUM"}),
    ));
    premium.add_item(altis_offer::OfferItem::new(
        "FAST_TRACK".to_string(),
        None,
        Some("FAST-TRACK".to_string()),
        "Fast Track Security".to_string(),
        Some("Skip the security line".to_string()),
        1700, // 2000 NUC with 15% discount
        req.passengers as i32,
        serde_json::json!({"category": "PREMIUM"}),
    ));
    offers.push(premium);
    
    // 3. Convert to response format
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
    // For now, return NOT_FOUND since we're not persisting offers yet
    // In full implementation, this would:
    // 1. Check Redis cache first
    // 2. Fall back to database
    // 3. Verify not expired
    
    Err(StatusCode::NOT_FOUND)
}

/// POST /v1/offers/:id/accept
/// Accept an offer and create an order
pub async fn accept_offer(
    State(state): State<AppState>,
    Path(offer_id): Path<Uuid>,
    Json(req): Json<AcceptOfferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Create a new order in PROPOSED status
    let order_id = Uuid::new_v4();
    
    // Return order details
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
    State(state): State<AppState>,
    Path(offer_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement offer expiry
    // 1. Mark offer as EXPIRED in database
    // 2. Remove from Redis cache
    // 3. Release any held inventory
    
    Ok(StatusCode::NO_CONTENT)
}
