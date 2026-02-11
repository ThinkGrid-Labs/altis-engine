extern crate altis_core;
use axum::{
    routing::get,
    Router,
    http::Method,
    extract::State,
    response::IntoResponse,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;

pub mod auth;
pub mod state;
pub mod search;
pub mod error;

pub use state::AppState;

// Renamed main's router setup to a function
pub fn app(state: AppState) -> Router {
    // CORS Middleware
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::USER_AGENT,
        ]);

    Router::new()
        .merge(auth::routes())
        .merge(search::routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state)
}

async fn rate_limit_middleware(
    State(state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let ip = addr.ip().to_string();
    let key = format!("ratelimit:{}", ip);
    
    match state.redis.check_rate_limit(&key, 100, 60).await {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err((axum::http::StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded")),
        Err(_) => Ok(next.run(req).await), // Fail open
    }
}
