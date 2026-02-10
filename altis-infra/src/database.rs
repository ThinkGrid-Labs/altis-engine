use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tracing::info;

pub mod config;

#[derive(Clone)]
pub struct DbClient {
    pub pool: Pool<Postgres>,
}

impl DbClient {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(connection_string)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        info!("Running database migrations...");
        sqlx::migrate!("../migrations")
            .run(&self.pool)
            .await?;
        info!("Migrations completed successfully.");
        Ok(())
    }
}
