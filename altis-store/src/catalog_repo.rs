use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use serde_json::Value;
use altis_core::repository::ProductRepository;

pub struct StoreProductRepository {
    pool: PgPool,
}

impl StoreProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Internal struct for type-safe querying
#[derive(sqlx::FromRow)]
struct ProductRow {
    id: Uuid,
    airline_id: Option<Uuid>,
    product_type: String,
    product_code: String,
    name: String,
    description: Option<String>,
    base_price_nuc: i32,
    #[allow(dead_code)]
    currency: Option<String>, 
    is_active: Option<bool>,
    margin_percentage: Option<f64>,
    metadata: Option<Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}


#[async_trait]
impl ProductRepository for StoreProductRepository {
    async fn create_product(
        &self,
        product: &Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let product_id = if let Some(id_str) = product["id"].as_str() {
            Uuid::parse_str(id_str)?
        } else {
            Uuid::new_v4()
        };

        let airline_id_str = product["airline_id"].as_str().ok_or("Missing airline_id")?;
        let airline_id = Uuid::parse_str(airline_id_str)?;

        let product_type = product["product_type"].as_str().unwrap_or("UNKNOWN");
        let product_code = product["product_code"].as_str().unwrap_or("UNKNOWN");
        let name = product["name"].as_str().unwrap_or("Unnamed Product");
        let description = product["description"].as_str();
        let base_price_nuc = product["base_price_nuc"].as_i64().unwrap_or(0) as i32;
        let is_active = product["is_active"].as_bool().unwrap_or(true);
        let metadata = &product["metadata"];

        sqlx::query!(
            r#"
            INSERT INTO products (id, airline_id, product_type, product_code, name, description, base_price_nuc, is_active, margin_percentage, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            product_id,
            airline_id,
            product_type,
            product_code,
            name,
            description,
            base_price_nuc,
            is_active,
            product["margin_percentage"].as_f64().unwrap_or(0.15),
            metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(product_id)
    }

    async fn get_product(
        &self,
        id: Uuid,
    ) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        // Use query_as! for strict typing
        let row = sqlx::query_as!(
            ProductRow,
            "SELECT id, airline_id, product_type, product_code, name, description, base_price_nuc, currency, is_active, margin_percentage::FLOAT8, metadata, created_at, updated_at FROM products WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let product_json = serde_json::json!({
                "id": row.id,
                "airline_id": row.airline_id,
                "product_type": row.product_type,
                "product_code": row.product_code,
                "name": row.name,
                "description": row.description,
                "base_price_nuc": row.base_price_nuc,
                "is_active": row.is_active,
                "metadata": row.metadata,
                "created_at": row.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                "updated_at": row.updated_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339())
            });
            return Ok(Some(product_json));
        }

        Ok(None)
    }

    async fn list_products(
        &self,
        airline_id: Uuid,
        product_type: Option<&str>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let products: Vec<ProductRow> = if let Some(pt) = product_type {
            sqlx::query_as!(
                ProductRow,
                "SELECT id, airline_id, product_type, product_code, name, description, base_price_nuc, currency, is_active, margin_percentage::FLOAT8, metadata, created_at, updated_at FROM products WHERE airline_id = $1 AND product_type = $2 ORDER BY name",
                airline_id,
                pt
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                ProductRow,
                "SELECT id, airline_id, product_type, product_code, name, description, base_price_nuc, currency, is_active, margin_percentage::FLOAT8, metadata, created_at, updated_at FROM products WHERE airline_id = $1 ORDER BY name",
                airline_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        let result = products.into_iter().map(|row| {
             serde_json::json!({
                "id": row.id,
                "airline_id": row.airline_id,
                "product_type": row.product_type,
                "product_code": row.product_code,
                "name": row.name,
                "description": row.description,
                "base_price_nuc": row.base_price_nuc,
                "is_active": row.is_active,
                "metadata": row.metadata,
                "created_at": row.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                "updated_at": row.updated_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339())
            })
        }).collect();

        Ok(result)
    }

    async fn update_product(
        &self,
        id: Uuid,
        product: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = product["name"].as_str().unwrap_or("Unnamed Product");
        let description = product["description"].as_str();
        let base_price_nuc = product["base_price_nuc"].as_i64().unwrap_or(0) as i32;
        let is_active = product["is_active"].as_bool().unwrap_or(true);
        let metadata = &product["metadata"];

        sqlx::query!(
            r#"
            UPDATE products 
            SET name = $1, description = $2, base_price_nuc = $3, is_active = $4, metadata = $5, updated_at = NOW()
            WHERE id = $6
            "#,
            name,
            description,
            base_price_nuc,
            is_active,
            metadata,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_product(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query!(
            "DELETE FROM products WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn get_airline_by_code(
        &self,
        code: &str,
    ) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query!(
            "SELECT id, code, name, country, status FROM airlines WHERE code = $1",
            code
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            return Ok(Some(serde_json::json!({
                "id": row.id,
                "code": row.code,
                "name": row.name,
                "country": row.country,
                "status": row.status
            })));
        }

        Ok(None)
    }
}
