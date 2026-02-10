use warp::Filter;
use std::sync::Arc;
use altis_infra::{RedisClient, EventProducer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use tokio::sync::broadcast;
use warp::sse::Event;
use futures_util::StreamExt;
use altis_domain::events::SeatHeldEvent;
use crate::auth::{with_auth, Claims};

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisClient>,
    pub kafka: Arc<EventProducer>,
    pub sse_tx: broadcast::Sender<SeatHeldEvent>,
    pub jwt_secret: String,
    pub business_rules: altis_infra::config::BusinessRules,
}

#[derive(Debug, Deserialize)]
struct CreateTripRequest {
    flight_ids: Vec<Uuid>,
    // user_id removed, extracted from token
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

pub fn routes(state: AppState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    let trip_hold = warp::path!("v1" / "holds" / "trip")
        .and(warp::post())
        .and(with_auth(state.jwt_secret.clone())) // Pass secret
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(create_trip_hold);

    let seat_hold = warp::path!("v1" / "holds" / "seat")
        .and(warp::post())
        .and(with_auth(state.jwt_secret.clone())) // Pass secret
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(create_seat_hold);

    let add_passenger = warp::path!("v1" / "holds" / "trip" / Uuid / "passengers")
        .and(warp::post())
        .and(with_auth(state.jwt_secret.clone()))
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(add_trip_passenger);

    let sse_stream = warp::path!("v1" / "flights" / Uuid / "stream")
        .and(warp::get())
        .and(with_auth(state.jwt_secret.clone())) // Require Auth
        .and(state_filter.clone())
        .map(|flight_id: Uuid, _claims: Claims, state: AppState| {
            let rx = state.sse_tx.subscribe();
            
            let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(move |result| {
                    let flight_id = flight_id.clone();
                    async move {
                        match result {
                            Ok(event) => {
                                if event.flight_id == flight_id {
                                    Some(Ok::<_, warp::Error>(
                                        Event::default()
                                            .event("seat_held")
                                            .data(serde_json::to_string(&event).unwrap())
                                    ))
                                } else {
                                    None
                                }
                            },
                            Err(_) => None,
                        }
                    }
                });

            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    trip_hold.or(seat_hold).or(sse_stream).or(add_passenger)
}

#[derive(Debug, Deserialize)]
struct AddPassengerRequest {
    first_name: String,
    last_name: String,
    passenger_type: String, // ADULT, CHILD
}

#[derive(Debug, Serialize)]
struct AddPassengerResponse {
    passenger_id: String,
}

async fn add_trip_passenger(trip_id: Uuid, claims: Claims, req: AddPassengerRequest, state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    // 1. Verify Owner
    let owner = state.redis.hget_trip_field(&trip_id.to_string(), "owner").await
        .map_err(|_| warp::reject::custom(InternalServerError))?
        .ok_or(warp::reject::custom(InternalServerError))?; // Or NotFound

    if owner != claims.sub {
        return Ok(warp::reply::with_status(warp::reply::json(&"Unauthorized"), warp::http::StatusCode::FORBIDDEN));
    }

    // 2. Generate Temp Pax ID
    let passenger_id = Uuid::new_v4().to_string();
    
    // 3. Serialize Passenger Data
    let pax_data = serde_json::to_string(&req).unwrap();
    
    // 4. Save to Hash: field "pax:{id}"
    let field = format!("pax:{}", passenger_id);
    state.redis.hset_trip_field(&trip_id.to_string(), &field, &pax_data).await
        .map_err(|_| warp::reject::custom(InternalServerError))?;
        
    Ok(warp::reply::json(&AddPassengerResponse { passenger_id }))
}

async fn create_trip_hold(claims: Claims, req: CreateTripRequest, state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    let trip_id = Uuid::new_v4();
    let ttl = state.business_rules.trip_hold_seconds; // Configurable
    
    // Validation: Must have at least one flight
    if req.flight_ids.is_empty() {
        return Err(warp::reject::custom(InternalServerError)); // Should return 400 Bad Request ideally
    }
    
    // Store user session (sub) map to trip
    // Key: trip:{trip_id} -> Value: { flight_ids: [...], user_id: claims.sub }
    // Serialization format: "flight_id1,flight_id2|user_id"
    
    let flight_ids_str = req.flight_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // Use Redis Hash for Session
    // 1. Set Flights
    state.redis.hset_trip_field(&trip_id.to_string(), "flights", &flight_ids_str).await
        .map_err(|_| warp::reject::custom(InternalServerError))?;
        
    // 2. Set Owner
    state.redis.hset_trip_field(&trip_id.to_string(), "owner", &claims.sub).await
        .map_err(|_| warp::reject::custom(InternalServerError))?;
        
    // 3. Set Status
    state.redis.hset_trip_field(&trip_id.to_string(), "status", "DRAFT").await
        .map_err(|_| warp::reject::custom(InternalServerError))?;

    // 4. Set Expiration (TTL)
    state.redis.exp_trip_key(&trip_id.to_string(), ttl as usize).await
        .map_err(|_| warp::reject::custom(InternalServerError))?;

    let expires_at = Utc::now().timestamp() + (ttl as i64);
    Ok(warp::reply::json(&CreateTripResponse { trip_id, expires_at }))
}

async fn create_seat_hold(claims: Claims, req: SeatHoldRequest, state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    // 1. Verify Owner and Flights using Hash
    let owner_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "owner").await.map_err(|_| warp::reject::custom(InternalServerError))?;
    let flights_opt = state.redis.hget_trip_field(&req.trip_id.to_string(), "flights").await.map_err(|_| warp::reject::custom(InternalServerError))?;

    let owner_id = match owner_opt {
        Some(v) => v,
        None => return Ok(warp::reply::with_status(warp::reply::json(&"Trip not found"), warp::http::StatusCode::NOT_FOUND)),
    };
    
    // 2. Verify Ownership
    if owner_id != claims.sub {
        return Ok(warp::reply::with_status(warp::reply::json(&"Unauthorized: Trip does not belong to you"), warp::http::StatusCode::FORBIDDEN));
    }
    
    // 3. Verify Flight
    let flight_ids_str = flights_opt.ok_or(warp::reject::custom(InternalServerError))?;
    let flight_ids_list: Vec<&str> = flight_ids_str.split(',').collect();
    let req_flight_id_str = req.flight_id.to_string();
    
    if !flight_ids_list.contains(&req_flight_id_str.as_str()) {
         return Ok(warp::reply::with_status(warp::reply::json(&"Flight not part of this trip"), warp::http::StatusCode::BAD_REQUEST));
    }
    
    let flight_id = req.flight_id;

    // 4. Try to lock seat
    let ttl = 900; // 15 minutes
    
    let locked = state.redis.acquire_seat_lock(&req.flight_id.to_string(), &req.seat_number, &req.trip_id.to_string(), ttl).await.map_err(|_| warp::reject::custom(InternalServerError))?;

    if locked {
        // 4. Publish Event
        let event = SeatHeldEvent {
            flight_id,
            seat_number: req.seat_number.clone(),
            trip_id: req.trip_id,
            held_at: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&event).unwrap();
        let _ = state.kafka.publish("holds.created", &req.flight_id.to_string(), &payload).await;

        let _ = state.sse_tx.send(event);

        Ok(warp::reply::with_status(warp::reply::json(&SeatHoldResponse { status: "HELD".to_string() }), warp::http::StatusCode::OK))
    } else {
        Ok(warp::reply::with_status(warp::reply::json(&"Seat already held"), warp::http::StatusCode::CONFLICT))
    }
}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}
