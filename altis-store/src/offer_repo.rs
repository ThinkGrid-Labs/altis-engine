use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use redis::AsyncCommands;
use serde_json::Value;
use std::sync::Arc;
use altis_core::repository::OfferRepository;

pub struct StoreOfferRepository {
    pool: PgPool,
    redis: Arc<redis::Client>,
}

impl StoreOfferRepository {
    pub fn new(pool: PgPool, redis: Arc<redis::Client>) -> Self {
        Self { pool, redis }
    }
}

// Internal struct for type-safe querying
#[derive(sqlx::FromRow)]
struct OfferItemRow {
    id: Uuid,
    #[allow(dead_code)] // fetched but maybe not used directly in JSON map depending on structure
    offer_id: Option<Uuid>, 
    product_id: Option<Uuid>,
    product_type: String,
    product_code: Option<String>,
    name: String,
    description: Option<String>,
    price_nuc: i32,
    quantity: Option<i32>,
    metadata: Option<Value>,
    #[allow(dead_code)] // schema has it
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}


#[async_trait]
impl OfferRepository for StoreOfferRepository {
    async fn save_offer(
        &self,
        offer: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let offer_id_str = offer["id"].as_str().ok_or("Missing offer ID")?;
        let offer_id = Uuid::parse_str(offer_id_str)?;
        
        let customer_id = offer["customer_id"].as_str();
        let airline_id_str = offer["airline_id"].as_str();
        let airline_id = if let Some(id) = airline_id_str {
            Some(Uuid::parse_str(id)?)
        } else {
            None
        };
        
        let search_context = &offer["search_context"];
        let total_nuc = offer["total_nuc"].as_i64().ok_or("Missing total_nuc")? as i32;
        let currency = offer["currency"].as_str().unwrap_or("NUC");
        let status = offer["status"].as_str().unwrap_or("ACTIVE");
        
        let expires_at_str = offer["expires_at"].as_str().ok_or("Missing expires_at")?;
        let expires_at = chrono::DateTime::parse_from_rfc3339(expires_at_str)?.with_timezone(&chrono::Utc);

        // 1. Save to Redis (Cache) - 15 minutes TTL
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(
            format!("offer:{}", offer_id),
            offer.to_string(),
            900
        ).await?;

        // 2. Save to Postgres (Persistent)
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, currency, status, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            offer_id,
            customer_id,
            airline_id,
            search_context,
            total_nuc,
            currency,
            status,
            expires_at
        )
        .execute(&mut *tx)
        .await?;

        // 3. Save items
        if let Some(items) = offer["items"].as_array() {
            for item in items {
                let item_id = Uuid::parse_str(item["id"].as_str().unwrap_or_default())?;
                let product_id_str = item["product_id"].as_str();
                let product_id = if let Some(id) = product_id_str { Some(Uuid::parse_str(id)?) } else { None };
                let product_type = item["product_type"].as_str().unwrap_or("UNKNOWN");
                let product_code = item["product_code"].as_str();
                let name = item["name"].as_str().unwrap_or("Unknown Item");
                let description = item["description"].as_str();
                let price_nuc = item["price_nuc"].as_i64().unwrap_or(0) as i32;
                let quantity = item["quantity"].as_i64().unwrap_or(1) as i32;
                let metadata = &item["metadata"];

                sqlx::query!(
                    r#"
                    INSERT INTO offer_items (id, offer_id, product_id, product_type, product_code, name, description, price_nuc, quantity, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    "#,
                    item_id,
                    offer_id,
                    product_id,
                    product_type,
                    product_code,
                    name,
                    description,
                    price_nuc,
                    quantity,
                    metadata
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_offer(
        &self,
        id: Uuid,
    ) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Try Redis first
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let cached: Option<String> = conn.get(format!("offer:{}", id)).await?;
        
        if let Some(json_str) = cached {
            return Ok(Some(serde_json::from_str(&json_str)?));
        }

        // 2. Fallback to Postgres
        let offer_row = sqlx::query!(
            "SELECT * FROM offers WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = offer_row {
            // Fetch items
            let items: Vec<OfferItemRow> = sqlx::query_as!(
                OfferItemRow,
                "SELECT id, offer_id, product_id, product_type, product_code, name, description, price_nuc, quantity, metadata, created_at FROM offer_items WHERE offer_id = $1",
                id
            )
            .fetch_all(&self.pool)
            .await?;

            let items_json: Vec<Value> = items.into_iter().map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "product_id": item.product_id,
                    "product_type": item.product_type,
                    "product_code": item.product_code,
                    "name": item.name,
                    "description": item.description,
                    "price_nuc": item.price_nuc,
                    "quantity": item.quantity,
                    "metadata": item.metadata,
                    // No created_at needed in OfferItem JSON usually, but we can include if needed
                    // "created_at": item.created_at.map(|t| t.to_rfc3339())
                })
            }).collect();

            let offer_json = serde_json::json!({
                "id": row.id,
                "customer_id": row.customer_id,
                "airline_id": row.airline_id,
                "search_context": row.search_context,
                "items": items_json,
                "total_nuc": row.total_nuc,
                "currency": row.currency,
                "status": row.status,
                "expires_at": row.expires_at.to_rfc3339(),
                "created_at": row.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
            });

            return Ok(Some(offer_json));
        }

        Ok(None)
    }

    async fn list_active_offers(
        &self,
        customer_id: &str,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query!(
            "SELECT id FROM offers WHERE customer_id = $1 AND status = 'ACTIVE' AND expires_at > NOW()",
            customer_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut offers = Vec::new();
        for row in rows {
            if let Some(offer) = self.get_offer(row.id).await? {
                offers.push(offer);
            }
        }
        Ok(offers)
    }

    async fn expire_offer(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Update DB
        sqlx::query!(
            "UPDATE offers SET status = 'EXPIRED' WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        // Remove from Redis
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: () = conn.del(format!("offer:{}", id)).await?;

        Ok(())
    }
}
