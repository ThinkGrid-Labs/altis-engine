use warp::Filter;
use std::sync::Arc;
use altis_infra::{RedisClient, DbClient, EventProducer, BookingRepository};
use altis_domain::booking::{CreateBookingRequest, BookingStatus};
use uuid::Uuid;
use serde::Serialize;
use tracing::info;
use crate::auth::{with_auth, Claims};

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisClient>,
    pub db: Arc<DbClient>,
    pub kafka: Arc<EventProducer>,
    pub jwt_secret: String,
}

#[derive(Debug, Serialize)]
struct BookingResponse {
    booking_id: Uuid,
    status: String,
}

pub fn routes(state: AppState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    warp::path!("v1" / "bookings" / "commit")
        .and(warp::post())
        .and(with_auth(state.jwt_secret.clone())) 
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(commit_booking)
}
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(commit_booking)
}

    // 1. Validate Trip Hold via Hash
    let owner_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "owner").await.map_err(|_| warp::reject::custom(InternalServerError))?;
    let flights_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "flights").await.map_err(|_| warp::reject::custom(InternalServerError))?;

    let owner_id = match owner_opt {
        Some(v) => v,
        None => return Ok(warp::reply::with_status(warp::reply::json(&"Trip expired or invalid"), warp::http::StatusCode::BAD_REQUEST)),
    };
    
    // 2. Verify Ownership
    if owner_id != claims.sub {
        return Ok(warp::reply::with_status(warp::reply::json(&"Unauthorized: Trip does not belong to you"), warp::http::StatusCode::FORBIDDEN));
    }

    let flight_ids_str = flights_opt.ok_or(warp::reject::custom(InternalServerError))?;
    let flight_ids: Vec<&str> = flight_ids_str.split(',').collect();
    let primary_flight_id = Uuid::parse_str(flight_ids[0]).map_err(|_| warp::reject::custom(InternalServerError))?;

    // 3. Start Transaction
    let mut tx = state.db.pool.begin().await.map_err(|_| warp::reject::custom(InternalServerError))?;

    let booking_id = Uuid::new_v4();
    
    // Create Booking (referencing primary flight for now)
    let _booking = BookingRepository::create_booking(
        &mut tx,
        booking_id,
        primary_flight_id,
        &req.user_email,
        10000 * (flight_ids.len() as i32), // Mock price logic
        "USD"
    ).await.map_err(|e| {
        info!("Failed to create booking: {}", e);
        warp::reject::custom(InternalServerError)
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
            warp::reject::custom(InternalServerError)
        })?;
    }

    // Confirm Booking
    BookingRepository::confirm_booking(&mut tx, booking_id).await.map_err(|e| {
        info!("Failed to confirm booking: {}", e);
        warp::reject::custom(InternalServerError)
    })?;

    // Commit Transaction
    tx.commit().await.map_err(|_| warp::reject::custom(InternalServerError))?;

    // 4. Publish Event
    let _ = state.kafka.publish("booking.confirmed", &booking_id.to_string(), &booking_id.to_string()).await;

    info!("Booking confirmed: {}", booking_id);

    Ok(warp::reply::json(&BookingResponse {
        booking_id,
        status: BookingStatus::CONFIRMED.to_string(),
    }))
}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}
