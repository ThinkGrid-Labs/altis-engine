extern crate altis_core;
use axum::{
    routing::{get, post},
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
pub mod offers;
pub mod orders;
pub mod admin;
pub mod middleware;

pub use state::AppState;

// ============================================================================
// Customer-Facing Routes (/v1/*)
// ============================================================================

fn customer_routes(state: AppState) -> Router<AppState> {
    let auth_routes = Router::new().nest("/auth", auth::routes());
    let public_search = search::routes(); // Keep search public? User said "this token use to search offers". So search needs token.
    
    Router::new()
        .merge(auth_routes)
        // Protected Routes (Offers, Orders, Search)
        .merge(
            Router::new()
                .merge(public_search) // Request: "this token use to search offers". So search MUST be protected.
                // Offers
                .route("/offers/search", post(offers::search_offers))
                .route("/offers/{id}", get(offers::get_offer).delete(offers::expire_offer))
                .route("/offers/{id}/accept", post(offers::accept_offer))
                
                // Orders
                .route("/orders", get(orders::list_orders))
                .route("/orders/{id}", get(orders::get_order))
                .route("/orders/{id}/pay", post(orders::pay_order))
                .route("/orders/{id}/customize", post(orders::customize_order))
                .route("/orders/{id}/fulfillment", get(orders::get_fulfillment))
                .route("/orders/{id}/cancel", post(orders::cancel_order))
                .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::customer_auth_middleware))
        )
}

// ============================================================================
// Admin Routes (/v1/admin/*)
// ============================================================================

fn admin_routes() -> Router<AppState> {
    Router::new()
        // Product Management
        .route("/airlines/{airline_id}/products", get(admin::list_products).post(admin::create_product))
        .route("/products/{id}", get(admin::get_product).put(admin::update_product).delete(admin::delete_product))
        
        // Pricing Rules
        .route("/airlines/{airline_id}/pricing-rules", get(admin::list_pricing_rules).post(admin::create_pricing_rule))
        .route("/pricing-rules/{id}", get(admin::get_pricing_rule).put(admin::update_pricing_rule).delete(admin::delete_pricing_rule))
        
        // Bundle Templates
        .route("/airlines/{airline_id}/bundles", get(admin::list_bundles).post(admin::create_bundle))
        .route("/bundles/{id}", get(admin::get_bundle).put(admin::update_bundle).delete(admin::delete_bundle))
}

// ============================================================================
// Main Application Router
// ============================================================================

pub fn app(state: AppState) -> Router {
    // CORS Middleware
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::USER_AGENT,
        ]);

    Router::new()
        // Customer routes at /v1/*
        .nest("/v1", customer_routes(state.clone()))
        
        // Admin routes at /v1/admin/*
        .nest("/v1/admin", admin_routes())
        
        // Health check
        .route("/health", get(health_check))
        
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state)
}

// ============================================================================
// Middleware
// ============================================================================

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

// ============================================================================
// Health Check
// ============================================================================

async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
