use axum::{
    extract::State,
    Json,
    routing::post,
    Router,

};
use altis_core::search::{FlightSearchRequest, FlightSearchResult};
use tracing::info;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/flights/search", post(search_flights))
}

use crate::error::AppError;

async fn search_flights(
    State(_state): State<AppState>,
    Json(req): Json<FlightSearchRequest>
) -> Result<Json<FlightSearchResult>, AppError> {
    // Mock flight search - return empty results for now
    // In production, this would query the flight repository
    info!("Flight search request for {} passenger(s)", req.passenger_count);
    
    let results = Vec::new(); // Empty results for mock

    Ok(Json(FlightSearchResult { legs: results }))
}
