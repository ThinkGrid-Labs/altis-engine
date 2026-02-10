use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    routing::post,
    Router,
    http::StatusCode,
};
use altis_infra::FlightRepository;
use altis_domain::search::{FlightSearchRequest, FlightSearchResult};
use tracing::info;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/flights/search", post(search_flights))
}

async fn search_flights(
    State(state): State<AppState>,
    Json(req): Json<FlightSearchRequest>
) -> impl IntoResponse {
    let mut results = Vec::new();

    for leg in req.legs {
        // Query repo for each leg
        match FlightRepository::search_flights(
            &state.db.pool,
            &state.redis, // Pass Redis
            &leg,
            req.passenger_count
        ).await {
            Ok(options) => results.push(options),
            Err(e) => {
                info!("Search failed: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Search failed").into_response();
            }
        }
    }

    Json(FlightSearchResult { legs: results }).into_response()
}
