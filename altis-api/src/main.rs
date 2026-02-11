use std::sync::Arc;
use std::net::SocketAddr;
use altis_api::{app, state::{AppState, AuthConfig}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "altis_api=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = altis_store::app_config::Config::load().expect("Failed to load config");
    tracing::info!("Starting Altis API on port {}", config.server.port);

    // Redis Connection
    let redis_client = altis_store::RedisClient::new(&config.redis.url)
        .await
        .expect("Failed to connect to Redis");
    let redis_arc = Arc::new(redis_client);

    // Kafka Connection
    let kafka_producer = altis_store::EventProducer::new(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");
    let kafka_arc = Arc::new(kafka_producer);

    // SSE Broadcast Channel
    let (sse_tx, _) = tokio::sync::broadcast::channel(100);

    let app_state = AppState {
        redis: redis_arc.clone(),
        kafka: kafka_arc,
        sse_tx,
        business_rules: config.business_rules.clone(),
        auth: AuthConfig {
            secret: config.auth.jwt_secret.clone(),
            expiration: config.auth.jwt_expiration_seconds,
        },
    };

    let app = app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await.unwrap();
}
