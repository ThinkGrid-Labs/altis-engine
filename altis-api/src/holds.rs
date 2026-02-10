use axum::{
    extract::{State, Path},
    Json,
    response::{IntoResponse, sse::{Event, Sse}},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use tokio_stream::StreamExt;
use altis_domain::events::SeatHeldEvent;
use crate::state::AppState;
use futures_util::stream::Stream;
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
struct CreateTripRequest {
    flight_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
struct CreateTripResponse {
    trip_id: Uuid,
    expires_at: i64,
}

#[derive(Debug, Deserialize)]
struct SeatHoldRequest {
    trip_id: Uuid,
    flight_id: Uuid,
    seat_number: String,
}

#[derive(Debug, Serialize)]
struct SeatHoldResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct AddPassengerRequest {
    first_name: String,
    last_name: String,
    passenger_type: String,
}

#[derive(Debug, Serialize)]
struct AddPassengerResponse {
    passenger_id: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/holds/trip", post(create_trip_hold))
        .route("/v1/holds/seat", post(create_seat_hold))
        .route("/v1/holds/trip/:trip_id/passengers", post(add_trip_passenger))
        .route("/v1/flights/:flight_id/stream", get(sse_stream))
}

async fn create_trip_hold(State(state): State<AppState>, Json(req): Json<CreateTripRequest>) -> impl IntoResponse {
    let trip_id = Uuid::new_v4();
    let ttl = state.business_rules.trip_hold_seconds;
    
    if req.flight_ids.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, "No flights provided").into_response();
    }
    
    let flight_ids_str = req.flight_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // TODO: Extract claims from context (requires Middleware)
    let owner_id = "guest"; // Placeholder, requires auth middleware

    let _ = state.redis.hset_trip_field(&trip_id.to_string(), "flights", &flight_ids_str).await;
    let _ = state.redis.hset_trip_field(&trip_id.to_string(), "owner", owner_id).await;
    let _ = state.redis.hset_trip_field(&trip_id.to_string(), "status", "DRAFT").await;
    let _ = state.redis.exp_trip_key(&trip_id.to_string(), ttl as usize).await;

    let expires_at = Utc::now().timestamp() + (ttl as i64);
    Json(CreateTripResponse { trip_id, expires_at }).into_response()
}

async fn create_seat_hold(State(state): State<AppState>, Json(req): Json<SeatHoldRequest>) -> impl IntoResponse {
    // Logic similar to before, simplified error handling for MVP refactor
    let ttl = state.business_rules.seat_hold_seconds;
    
    match state.redis.acquire_seat_lock(&req.flight_id.to_string(), &req.seat_number, &req.trip_id.to_string(), ttl).await {
        Ok(true) => {
            let event = SeatHeldEvent {
                flight_id: req.flight_id,
                seat_number: req.seat_number.clone(),
                trip_id: req.trip_id,
                held_at: Utc::now().timestamp(),
            };
            let payload = serde_json::to_string(&event).unwrap();
            let _ = state.kafka.publish("holds.created", &req.flight_id.to_string(), &payload).await;
            let _ = state.sse_tx.send(event);
            
            Json(SeatHoldResponse { status: "HELD".to_string() }).into_response()
        },
        _ => (axum::http::StatusCode::CONFLICT, "Seat already held").into_response()
    }
}

async fn add_trip_passenger(State(state): State<AppState>, Path(trip_id): Path<Uuid>, Json(req): Json<AddPassengerRequest>) -> impl IntoResponse {
    let passenger_id = Uuid::new_v4().to_string();
    let pax_data = serde_json::to_string(&req).unwrap();
    let field = format!("pax:{}", passenger_id);
    
    let _ = state.redis.hset_trip_field(&trip_id.to_string(), &field, &pax_data).await;
        
    Json(AddPassengerResponse { passenger_id }).into_response()
}

async fn sse_stream(State(state): State<AppState>, Path(flight_id): Path<Uuid>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.sse_tx.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |result| {
            let flight_id = flight_id.clone();
            async move {
                match result {
                    Ok(event) if event.flight_id == flight_id => {
                        Some(Ok(Event::default().data(serde_json::to_string(&event).unwrap())))
                    },
                    _ => None
                }
            }
        });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
