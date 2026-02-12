use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use serde_json::Value;
use altis_core::repository::OrderRepository;

pub struct StoreOrderRepository {
    pool: PgPool,
}

impl StoreOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Internal structs for type-safe querying
#[derive(sqlx::FromRow)]
struct OrderRow {
    id: Uuid,
    customer_id: String,
    customer_email: Option<String>,
    offer_id: Option<Uuid>,
    airline_id: Option<Uuid>,
    status: String,
    total_nuc: i32,
    currency: Option<String>,
    payment_method: Option<String>,
    payment_reference: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    id: Uuid,
    #[allow(dead_code)]
    order_id: Option<Uuid>,
    product_id: Option<Uuid>,
    product_type: String,
    product_code: Option<String>,
    name: String,
    description: Option<String>,
    price_nuc: i32,
    quantity: Option<i32>,
    status: Option<String>,
    metadata: Option<Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct FulfillmentRow {
    id: Uuid,
    #[allow(dead_code)]
    order_id: Option<Uuid>,
    order_item_id: Option<Uuid>,
    fulfillment_type: String,
    barcode: Option<String>,
    qr_code_data: Option<String>,
    delivery_method: Option<String>,
    delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
impl OrderRepository for StoreOrderRepository {
    async fn create_order(
        &self,
        order: &Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let order_id = if let Some(id_str) = order["id"].as_str() {
            Uuid::parse_str(id_str)?
        } else {
            Uuid::new_v4()
        };

        let customer_id = order["customer_id"].as_str().ok_or("Missing customer_id")?;
        let customer_email = order["customer_email"].as_str();
        
        let offer_id_str = order["offer_id"].as_str();
        let offer_id = if let Some(id) = offer_id_str { Some(Uuid::parse_str(id)?) } else { None };
        
        let airline_id_str = order["airline_id"].as_str();
        let airline_id = if let Some(id) = airline_id_str { Some(Uuid::parse_str(id)?) } else { None };

        let status = order["status"].as_str().unwrap_or("PROPOSED");
        let total_nuc = order["total_nuc"].as_i64().unwrap_or(0) as i32;
        let currency = order["currency"].as_str().unwrap_or("NUC");
        let payment_method = order["payment_method"].as_str();
        let payment_reference = order["payment_reference"].as_str();

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO orders (id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            order_id,
            customer_id,
            customer_email,
            offer_id,
            airline_id,
            status,
            total_nuc,
            currency,
            payment_method,
            payment_reference
        )
        .execute(&mut *tx)
        .await?;

        if let Some(items) = order["items"].as_array() {
            for item in items {
                let item_id = if let Some(id_str) = item["id"].as_str() {
                    Uuid::parse_str(id_str)?
                } else {
                    Uuid::new_v4()
                };
                
                let product_id_str = item["product_id"].as_str();
                let product_id = if let Some(id) = product_id_str { Some(Uuid::parse_str(id)?) } else { None };
                
                let product_type = item["product_type"].as_str().unwrap_or("UNKNOWN");
                let product_code = item["product_code"].as_str();
                let name = item["name"].as_str().unwrap_or("Unknown Item");
                let description = item["description"].as_str();
                let price_nuc = item["price_nuc"].as_i64().unwrap_or(0) as i32;
                let quantity = item["quantity"].as_i64().unwrap_or(1) as i32;
                let item_status = item["status"].as_str().unwrap_or("ACTIVE");
                let metadata = &item["metadata"];

                sqlx::query!(
                    r#"
                    INSERT INTO order_items (id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    "#,
                    item_id,
                    order_id,
                    product_id,
                    product_type,
                    product_code,
                    name,
                    description,
                    price_nuc,
                    quantity,
                    item_status,
                    metadata
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(order_id)
    }

    async fn get_order(
        &self,
        id: Uuid,
    ) -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let order_row = sqlx::query_as!(
            OrderRow,
            "SELECT id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference, created_at, updated_at FROM orders WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = order_row {
            let items_rows: Vec<OrderItemRow> = sqlx::query_as!(
                OrderItemRow,
                "SELECT id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, metadata, created_at, updated_at FROM order_items WHERE order_id = $1",
                id
            )
            .fetch_all(&self.pool)
            .await?;

            let items: Vec<Value> = items_rows.into_iter().map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "product_id": item.product_id,
                    "product_type": item.product_type,
                    "product_code": item.product_code,
                    "name": item.name,
                    "description": item.description,
                    "price_nuc": item.price_nuc,
                    "quantity": item.quantity,
                    "status": item.status,
                    "metadata": item.metadata,
                    "created_at": item.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                    "updated_at": item.updated_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339())
                })
            }).collect();

            let fulfillment_rows: Vec<FulfillmentRow> = sqlx::query_as!(
                FulfillmentRow,
                "SELECT id, order_id, order_item_id, fulfillment_type, barcode, qr_code_data, delivery_method, delivered_at, created_at FROM fulfillment WHERE order_id = $1",
                id
            )
            .fetch_all(&self.pool)
            .await?;

             let fulfillment: Vec<Value> = fulfillment_rows.into_iter().map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "order_item_id": f.order_item_id,
                    "fulfillment_type": f.fulfillment_type,
                    "barcode": f.barcode,
                    "qr_code_data": f.qr_code_data,
                    "delivery_method": f.delivery_method,
                    "delivered_at": f.delivered_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                    "created_at": f.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339())
                })
            }).collect();

            let order_json = serde_json::json!({
                "id": row.id,
                "customer_id": row.customer_id,
                "customer_email": row.customer_email,
                "offer_id": row.offer_id,
                "airline_id": row.airline_id,
                "status": row.status,
                "total_nuc": row.total_nuc,
                "currency": row.currency,
                "payment_method": row.payment_method,
                "payment_reference": row.payment_reference,
                "items": items,
                "fulfillment": fulfillment,
                "created_at": row.created_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
                "updated_at": row.updated_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339())
            });

            return Ok(Some(order_json));
        }

        Ok(None)
    }

    async fn update_order_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query!(
            "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
            status,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: &Value,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let item_id = Uuid::new_v4();
        
        let product_id_str = item["product_id"].as_str();
        let product_id = if let Some(id) = product_id_str { Some(Uuid::parse_str(id)?) } else { None };
        
        let product_type = item["product_type"].as_str().unwrap_or("UNKNOWN");
        let product_code = item["product_code"].as_str();
        let name = item["name"].as_str().unwrap_or("Unknown Item");
        let description = item["description"].as_str();
        let price_nuc = item["price_nuc"].as_i64().unwrap_or(0) as i32;
        let quantity = item["quantity"].as_i64().unwrap_or(1) as i32;
        let status = "ACTIVE";
        let metadata = &item["metadata"];

        sqlx::query!(
            r#"
            INSERT INTO order_items (id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            item_id,
            order_id,
            product_id,
            product_type,
            product_code,
            name,
            description,
            price_nuc,
            quantity,
            status,
            metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(item_id)
    }

    async fn list_orders(
        &self,
        customer_id: &str,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query!(
            "SELECT id FROM orders WHERE customer_id = $1 ORDER BY created_at DESC",
            customer_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            if let Some(order) = self.get_order(row.id).await? {
                orders.push(order);
            }
        }
        Ok(orders)
    }

    async fn create_fulfillment(
        &self,
        order_id: Uuid,
        order_item_id: Uuid,
        fulfillment_type: &str,
        barcode: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let fulfillment_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO fulfillment (id, order_id, order_item_id, fulfillment_type, barcode)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            fulfillment_id,
            order_id,
            order_item_id,
            fulfillment_type,
            barcode
        )
        .execute(&self.pool)
        .await?;

        Ok(fulfillment_id)
    }
}
