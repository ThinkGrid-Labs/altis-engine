use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    routing::post,
    Router,
    http::StatusCode,
};
use altis_domain::search::{FlightSearchRequest, FlightSearchResult};
use tracing::info;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/flights/search", post(search_flights))
}

use crate::error::AppError;

async fn search_flights(
    State(state): State<AppState>,
    Json(req): Json<FlightSearchRequest>
) -> Result<Json<FlightSearchResult>, AppError> {
    let mut results = Vec::new();

    for leg in req.legs {
        // Query repo for each leg
        match state.flight_repo.search_flights(
            &leg,
            req.passenger_count
        ).await {
            Ok(options) => results.push(options),
            Err(e) => {
                info!("Search failed: {}", e);
                return Err(AppError::InternalServerError("Search failed".to_string()));
            }
        }
    }

    Ok(Json(FlightSearchResult { legs: results }))
}
