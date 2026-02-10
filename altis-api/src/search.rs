use warp::Filter;
use std::sync::Arc;
use altis_infra::{DbClient, FlightRepository};
use altis_domain::search::{FlightSearchRequest, FlightSearchResult}; // Removed FlightOption unused import if not used explicitly
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbClient>,
    pub redis: Arc<altis_infra::RedisClient>,
}

pub fn routes(state: AppState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    warp::path!("v1" / "flights" / "search")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(search_flights)
}

async fn search_flights(req: FlightSearchRequest, state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    let mut results = Vec::new();

    for leg in req.legs {
        // Query repo for each leg
        let options = FlightRepository::search_flights(
            &state.db.pool,
            &state.redis, // Pass Redis
            &leg,
            req.passenger_count
        ).await.map_err(|e| {
            info!("Search failed: {}", e);
            warp::reject::custom(InternalServerError)
        })?;
        
        results.push(options);
    }

    Ok(warp::reply::json(&FlightSearchResult { legs: results }))
}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}
