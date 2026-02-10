use std::sync::Arc;
use warp::Filter;

mod holds;
mod bookings;
mod auth;

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
    
    // Kafka Connection
    let kafka_producer = altis_infra::EventProducer::new(&config.kafka.brokers).expect("Failed to create Kafka producer");

    // SSE Broadcast Channel
    let (sse_tx, _) = tokio::sync::broadcast::channel(100);

    // Load Dynamic Business Rules
    let dynamic_rules = match db_client.fetch_business_rules(config.business_rules.clone()).await {
        Ok(rules) => {
            tracing::info!("Loaded dynamic business rules from DB");
            rules
        },
        Err(e) => {
            tracing::warn!("Failed to load business rules from DB, using config defaults: {}", e);
            config.business_rules.clone()
        }
    };

    let holds_state = holds::AppState {
        redis: Arc::new(redis_client.clone()),
        kafka: Arc::new(kafka_producer.clone()),
        sse_tx,
        jwt_secret: config.auth.jwt_secret.clone(),
        business_rules: dynamic_rules.clone(),
    };

    let bookings_state = bookings::AppState {
        redis: Arc::new(redis_client),
        db: db_arc,
        kafka: Arc::new(kafka_producer),
        jwt_secret: config.auth.jwt_secret.clone(),
    };

mod auth;
mod search;
mod worker;
mod summary_handler;

    // Start Availability Worker (Background Task)
    let db_for_worker = db_arc.clone();
    let redis_for_worker = Arc::new(redis_client.clone());
    let brokers = config.kafka.brokers.clone();
    
    tokio::spawn(async move {
        worker::start_availability_worker(brokers, "altis-worker-group".to_string(), db_for_worker, redis_for_worker).await;
    });

    // Routes
    let routes = holds::routes(holds_state)
        .or(bookings::routes(bookings_state))
        .or(auth::routes(config.auth.jwt_secret, config.auth.jwt_expiration_seconds))
        .or(search::routes(search::AppState { db: db_arc.clone(), redis: Arc::new(redis_client) })); // Pass Redis to Search

    warp::serve(routes).run(([0, 0, 0, 0], config.server.port)).await;
}
