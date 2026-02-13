use std::sync::Arc;
use std::net::SocketAddr;
use altis_api::{app, state::{AppState, AuthConfig, ResiliencyState}};
use altis_api::middleware::resiliency::CircuitBreaker;
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

    // Database Pool
    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .expect("Failed to connect to Postgres");

    // Run Migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // Repositories
    let offer_repo = Arc::new(altis_store::StoreOfferRepository::new(pool.clone(), Arc::new(redis_arc.get_client())));
    let order_repo = Arc::new(altis_store::StoreOrderRepository::new(pool.clone()));
    let catalog_repo = Arc::new(altis_store::StoreProductRepository::new(pool.clone()));

    // AI/Telemetry
    let telemetry = Arc::new(altis_offer::events::OfferTelemetry::new(&config.kafka.brokers, "offers"));
    
    let ml_client = if let Some(url) = &config.ranking.ml_service_url {
        match tonic::transport::Endpoint::from_shared(url.clone()) {
            Ok(endpoint) => {
                match endpoint.connect().await {
                    Ok(channel) => {
                        tracing::info!("Connected to ML Ranking service at {}", url);
                        Some(altis_offer::ai_ranker::ranking::ranking_service_client::RankingServiceClient::new(channel))
                    },
                    Err(e) => {
                        tracing::error!("Failed to connect to ML service at {}: {}", url, e);
                        None
                    }
                }
            },
            Err(e) => {
                tracing::error!("Invalid ML service URL {}: {}", url, e);
                None
            }
        }
    } else {
        None
    };

    let ranker = Arc::new(tokio::sync::Mutex::new(altis_offer::ai_ranker::OfferRanker::new(
        config.ranking.clone(),
        Some(telemetry.clone()),
        ml_client,
    )));

    // Payment Orchestration
    let payment_adapter = Arc::new(altis_order::orchestrator::MockPaymentAdapter);
    let payment_orchestrator = Arc::new(altis_order::orchestrator::PaymentOrchestrator::new(payment_adapter));

    // One Identity
    let one_id_resolver = Arc::new(altis_core::identity::MockOneIdResolver);

    // Resiliency
    let resiliency = Arc::new(ResiliencyState {
        payment_cb: CircuitBreaker::new("PaymentGateway", 3, std::time::Duration::from_secs(30)),
        ndc_cb: CircuitBreaker::new("NDCAPI", 5, std::time::Duration::from_secs(60)),
    });

    let app_state = AppState {
        redis: redis_arc,
        kafka: kafka_arc,
        sse_tx,
        business_rules: config.business_rules.clone(),
        auth: AuthConfig {
            secret: config.auth.jwt_secret.clone(),
            expiration: config.auth.jwt_expiration_seconds,
        },
        offer_repo,
        order_repo,
        catalog_repo,
        telemetry,
        ranker,
        payment_orchestrator,
        one_id_resolver,
        resiliency,
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
