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
pub mod finance;
pub mod middleware;
use crate::middleware::resiliency::circuit_breaker_middleware;
pub mod webhooks;
pub mod v1 {
    pub mod ndc;
    pub mod oneorder;
}

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
                .route("/orders/{id}/payment-intent", post(orders::initialize_payment_intent))
                .route("/orders/{id}/reshop", post(orders::reshop_order))
                .route("/orders/{id}/customize", post(orders::customize_order))
                .route("/orders/{id}/fulfillment", get(orders::get_fulfillment))
                .route("/orders/{id}/cancel", post(orders::cancel_order))
                .route("/orders/{id}/accept-reaccommodation", post(orders::accept_reaccommodation))
                .route("/orders/{id}/involuntary-refund", post(orders::involuntary_refund))

                // Fulfillment / Service Delivery
                .route("/fulfillment/{barcode}/consume", post(orders::consume_fulfillment))
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

        // Disruption Management
        .route("/disruptions", post(admin::trigger_disruption))
        
        // Finance / Settlement
        .route("/finance/orders/{id}/ledger", get(finance::get_order_ledger))
        .route("/finance/airlines/{id}/settlement", get(finance::get_airline_settlement))
        .route("/finance/airlines/{id}/export/swo", get(finance::export_swo))
        .route("/finance/airlines/{id}/export/legacy", get(finance::export_legacy))
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
        
        // Webhooks
        .route("/v1/webhooks/payments/stripe", post(webhooks::handle_stripe_webhook))

        // Standardized IATA Interfaces
        .route("/v1/ndc/airshopping", post(v1::ndc::air_shopping))
        .route("/v1/oneorder/{id}", get(v1::oneorder::order_retrieve))

        // Health check
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(state.clone(), circuit_breaker_middleware))
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

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    use prometheus::{Encoder, TextEncoder, Registry, Gauge, Opts};
    
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let registry = Registry::new();
    
    // Payment Circuit Breaker Gauge
    let payment_failures = Gauge::with_opts(Opts::new("altis_payment_cb_failures", "Failure count for payment circuit breaker")).unwrap();
    payment_failures.set(state.resiliency.payment_cb.failure_count.load(std::sync::atomic::Ordering::SeqCst) as f64);
    
    // NDC Circuit Breaker Gauge
    let ndc_failures = Gauge::with_opts(Opts::new("altis_ndc_cb_failures", "Failure count for NDC circuit breaker")).unwrap();
    ndc_failures.set(state.resiliency.ndc_cb.failure_count.load(std::sync::atomic::Ordering::SeqCst) as f64);

    registry.register(Box::new(payment_failures.clone())).unwrap();
    registry.register(Box::new(ndc_failures.clone())).unwrap();
    
    encoder.encode(&registry.gather(), &mut buffer).unwrap();
    
    String::from_utf8(buffer).unwrap()
}
