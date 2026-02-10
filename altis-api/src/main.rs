use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    http::{Method, HeaderValue},
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use crate::state::AppState;

mod holds;
mod bookings;
mod auth;
mod state;
mod search;
mod worker;
mod summary_handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let config = altis_infra::config::Config::load().expect("Failed to load configuration");
    println!("Starting Altis API on port {}", config.server.port);
    println!("Database URL: {}", config.database.url);

    // DB Connection
    let db = altis_infra::DbClient::new(&config.database.url).await.expect("Failed to connect to database");
    db.migrate().await.expect("Failed to run migrations");
    let db_arc = Arc::new(db);

    // Redis Connection
    let redis_client = altis_infra::RedisClient::new(&config.redis.url).await.expect("Failed to connect to Redis");
    let redis_arc = Arc::new(redis_client.clone());
    
    // Kafka Connection
    let kafka_producer = altis_infra::EventProducer::new(&config.kafka.brokers).expect("Failed to create Kafka producer");
    let kafka_arc = Arc::new(kafka_producer);

    // SSE Broadcast Channel
    let (sse_tx, _) = tokio::sync::broadcast::channel(100);

    // Load Dynamic Business Rules
    let dynamic_rules = match db_arc.fetch_business_rules(config.business_rules.clone()).await {
        Ok(rules) => {
            tracing::info!("Loaded dynamic business rules from DB");
            rules
        },
        Err(e) => {
            tracing::warn!("Failed to load business rules from DB, using config defaults: {}", e);
            config.business_rules.clone()
        }
    };

    let app_state = AppState {
        db: db_arc.clone(),
        redis: redis_arc.clone(),
        kafka: kafka_arc.clone(),
        sse_tx: sse_tx.clone(),
        jwt_secret: config.auth.jwt_secret.clone(),
        jwt_expiration: config.auth.jwt_expiration_seconds,
        business_rules: dynamic_rules.clone(),
    };

    // Start Availability Worker (Background Task)
    let db_for_worker = db_arc.clone();
    let redis_for_worker = redis_arc.clone();
    let brokers = config.kafka.brokers.clone();
    
    tokio::spawn(async move {
        worker::start_availability_worker(brokers, "altis-worker-group".to_string(), db_for_worker, redis_for_worker).await;
    });

    // CORS Middleware
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::USER_AGENT,
        ]);

    // Build Router
    let app = Router::new()
        .merge(auth::routes())
        .merge(holds::routes())
        .merge(bookings::routes())
        .merge(search::routes())
        .route("/v1/trips/:trip_id/summary", get(summary_handler::get_trip_summary))
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), rate_limit_middleware))
        .with_state(app_state);

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    axum::serve(listener, app).await.unwrap();
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
