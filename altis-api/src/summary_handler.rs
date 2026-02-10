use axum::{
    extract::{State, Path},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::Serialize;
use uuid::Uuid;
use crate::state::AppState;

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

use crate::error::AppError;

pub async fn get_trip_summary(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>
) -> Result<Json<TripSummaryResponse>, AppError> {
    // 1. Get Session Data
    let owner_opt = state.redis.hget_trip_field(&trip_id.to_string(), "owner").await.ok().flatten();
    
    let _owner_id = match owner_opt {
        Some(v) => v,
        None => return Err(AppError::NotFoundError("Trip not found".to_string())),
    };
    
    // Auth check skipped for MVP refactor, should extract Claims from request extensions
    // if owner_id != claims.sub { return 403 }

    let flights_opt = state.redis.hget_trip_field(&trip_id.to_string(), "flights").await.ok().flatten();
    let flight_ids_str = match flights_opt {
        Some(s) => s,
        None => return Err(AppError::InternalServerError("Invalid Trip Data".to_string())),
    };
    let flight_ids: Vec<&str> = flight_ids_str.split(',').collect();

    // 2. Count Passengers
    let passenger_count = 1; 

    // 3. Calculate Base Fare
    let base_fare = 100.0 * (flight_ids.len() as f64); 

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
                multiplier = 1.0;
                adjustment = 0.0;
            }
        }
    }
    
    if let Some(end_str) = &state.business_rules.sale_end {
        if let Ok(end) = chrono::DateTime::parse_from_rfc3339(end_str) {
            if now > end.with_timezone(&chrono::Utc) {
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
    let seats_cost = 0.0;

    let total = final_base_fare + taxes + fees + seats_cost;

    Ok(Json(TripSummaryResponse {
        trip_id,
        breakdown: CostBreakdown {
            base_fare: final_base_fare,
            taxes,
            fees,
            seats: seats_cost,
            passengers: passenger_count,
        },
        total_amount: total,
        currency: "USD".to_string(),
    }))
}
