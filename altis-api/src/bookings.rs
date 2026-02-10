use axum::{
    extract::{State, Json},
    routing::{post},
    Router,
};
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum::TypedHeader;
use std::sync::Arc;
use altis_infra::BookingRepository;
use altis_domain::booking::{CreateBookingRequest, BookingStatus};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::info;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::state::AppState;
use crate::error::AppError;

#[derive(Debug, Serialize)]
struct BookingResponse {
    booking_id: Uuid,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/bookings/commit", post(commit_booking))
}

async fn commit_booking(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(req): Json<CreateBookingRequest>
) -> Result<Json<BookingResponse>, AppError> {

    // Decode Token (Manually for now, ideally via Middleware)
    let token_data = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(state.auth.secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::AuthenticationError(e.to_string()))?;

    let claims = token_data.claims;

    // 1. Validate Trip Hold via Hash
    let owner_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "owner").await.map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let flights_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "flights").await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let owner_id = match owner_opt {
        Some(v) => v,
        None => return Err(AppError::validation_error("Trip expired or invalid".to_string())),
    };
    
    // 2. Verify Ownership
    if owner_id != claims.sub {
        return Err(AppError::AuthorizationError("Unauthorized: Trip does not belong to you".to_string()));
    }

    let flight_ids_str = flights_opt.ok_or(AppError::InternalServerError("Missing flight data".to_string()))?;
    let flight_ids: Vec<&str> = flight_ids_str.split(',').collect();
    let primary_flight_id = Uuid::parse_str(flight_ids[0]).map_err(|_| AppError::InternalServerError("Invalid flight ID".to_string()))?;

    // 3. Start Transaction
    let mut tx = state.db.pool.begin().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let booking_id = Uuid::new_v4();
    
    // Create Booking
    let _booking = BookingRepository::create_booking(
        &mut tx,
        booking_id,
        primary_flight_id,
        &req.user_email,
        10000 * (flight_ids.len() as i32), // Mock price logic
        "USD"
    ).await.map_err(|e| {
        info!("Failed to create booking: {}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    // Add Passengers & Seats
    for p in req.passengers {
        BookingRepository::add_passenger(
            &mut tx,
            booking_id,
            &p.first_name,
            &p.last_name,
            &p.seats
        ).await.map_err(|e| {
            info!("Failed to add passenger: {}", e);
            AppError::InternalServerError(e.to_string())
        })?;
    }

    // Confirm Booking
    BookingRepository::confirm_booking(&mut tx, booking_id).await.map_err(|e| {
        info!("Failed to confirm booking: {}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    // Commit Transaction
    tx.commit().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // 4. Publish Event
    let _ = state.kafka.publish("booking.confirmed", &booking_id.to_string(), &booking_id.to_string()).await;

    info!("Booking confirmed: {}", booking_id);

    Ok(Json(BookingResponse {
        booking_id,
        status: BookingStatus::CONFIRMED.to_string(),
    }))
}
