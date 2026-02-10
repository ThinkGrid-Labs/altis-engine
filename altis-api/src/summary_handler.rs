use warp::Filter;
use std::sync::Arc;
use altis_infra::{DbClient, RedisClient};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{with_auth, Claims};
use crate::holds::AppState; // Reuse Holds State which has BusinessRules

#[derive(Debug, Serialize)]
struct TripSummaryResponse {
    trip_id: Uuid,
    breakdown: CostBreakdown,
    total_amount: f64,
    currency: String,
}

#[derive(Debug, Serialize)]
struct CostBreakdown {
    base_fare: f64,
    taxes: f64,
    fees: f64,
    seats: f64,
    passengers: usize,
}

pub async fn get_trip_summary(trip_id: Uuid, claims: Claims, state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    // 1. Get Session Data
    let owner_opt = state.redis.hget_trip_field(&trip_id.to_string(), "owner").await.map_err(|_| warp::reject::custom(InternalServerError))?;
    
    let owner_id = match owner_opt {
        Some(v) => v,
        None => return Ok(warp::reply::with_status(warp::reply::json(&"Trip not found"), warp::http::StatusCode::NOT_FOUND)),
    };
    
    if owner_id != claims.sub {
        return Ok(warp::reply::with_status(warp::reply::json(&"Unauthorized"), warp::http::StatusCode::FORBIDDEN));
    }

    let flights_opt = state.redis.hget_trip_field(&trip_id.to_string(), "flights").await.map_err(|_| warp::reject::custom(InternalServerError))?;
    let flight_ids_str = flights_opt.ok_or(warp::reject::custom(InternalServerError))?;
    let flight_ids: Vec<&str> = flight_ids_str.split(',').collect();

    // 2. Count Passengers (Mock logic for Phase 11 MVP: count keys starting with "pax:" via SCAN, or assume 1 if missing?)
    // Real implementation needs HGETALL or HSCAN.
    // For MVP, we will assume 1 passenger if none added, OR requires "passenger_count" field in Hash if we want speed.
    // Let's assume we store "pax_count" in Hash when adding passengers? No, that's redundant.
    // Let's implement `hgetall` in repo properly later.
    // FALLBACK: For now, default to 1 passenger.
    let passenger_count = 1; 

    // 3. Calculate Base Fare
    // Fetch prices from DB for all flights
    // SUM(price * pax)
    let mut base_fare = 0.0;
    
    // We reuse DbClient from another state? Holds State needs DB access?
    // Holds State currently has Redis, Kafka. NO DB.
    // We need to inject DB into Holds State or create a new `Trip` service.
    // Recommendation: Add DB to Holds State.
    
    // MOCK PRICE for "Holds" service separation (or we update Holds State).
    // Let's update Holds State to include DB in next step.
    // Assuming DB is available:
    /*
    for fid in flight_ids {
        let price = sqlx::query!("SELECT base_price_amount FROM flights WHERE id = $1", Uuid::parse_str(fid).unwrap())
            .fetch_one(&state.db.pool).await?.base_price_amount;
        base_fare += (price as f64) * (passenger_count as f64);
    }
    */
    base_fare = 100.0 * (flight_ids.len() as f64); // Mock Price $100 per flight

    // 4. Calculate Taxes & Fees
    let tax_rate = state.business_rules.tax_rate;
    let booking_fee = state.business_rules.booking_fee;
    let mut multiplier = state.business_rules.pricing_multiplier;
    let mut adjustment = state.business_rules.pricing_adjustment;
    
    // Check Configured Date Range for Sale
    let now = chrono::Utc::now();
    
    if let Some(start_str) = &state.business_rules.sale_start {
        if let Ok(start) = chrono::DateTime::parse_from_rfc3339(start_str) {
            if now < start.with_timezone(&chrono::Utc) {
                // Sale hasn't started
                multiplier = 1.0;
                adjustment = 0.0;
            }
        }
    }
    
    if let Some(end_str) = &state.business_rules.sale_end {
        if let Ok(end) = chrono::DateTime::parse_from_rfc3339(end_str) {
            if now > end.with_timezone(&chrono::Utc) {
                // Sale ended
                multiplier = 1.0;
                adjustment = 0.0;
            }
        }
    }

    // Apply Pricing Rules (Sale / Surcharge)
    let adjusted_base_fare = (base_fare * multiplier) + adjustment;
    let final_base_fare = if adjusted_base_fare < 0.0 { 0.0 } else { adjusted_base_fare };
    
    let taxes = final_base_fare * tax_rate; 
    let fees = booking_fee;
    
    // 5. Ancillaries (Seats)
    // Needs to scan session for "seat:..." keys.
    // Mock for MVP:
    let seats_cost = 0.0;

    let total = final_base_fare + taxes + fees + seats_cost;

    Ok(warp::reply::json(&TripSummaryResponse {
        trip_id,
        breakdown: CostBreakdown {
            base_fare: final_base_fare, // Return adjusted fare
            taxes,
            fees,
            seats: seats_cost,
            passengers: passenger_count,
        },
        total_amount: total,
        currency: "USD".to_string(),
    }))
}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}
