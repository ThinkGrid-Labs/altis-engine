use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use altis_core::repository::OrderRepository;

pub struct PostgresOrderRepository {
    pub pool: PgPool,
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn create_order(
        &self,
        order: &altis_order::Order,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        // Insert order
        sqlx::query!(
            r#"
            INSERT INTO orders (id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            order.id,
            order.customer_id,
            order.customer_email.as_deref(),
            order.offer_id,
            order.airline_id,
            order.status.to_string(),
            order.total_nuc,
            order.currency,
            order.payment_method.as_deref(),
            order.payment_reference.as_deref()
        )
        .execute(&self.pool)
        .await?;
        
        // Insert order items
        for item in &order.items {
            self.add_order_item(order.id, item).await?;
        }
        
        Ok(order.id)
    }
    
    async fn get_order(
        &self,
        id: Uuid,
    ) -> Result<Option<altis_order::Order>, Box<dyn std::error::Error + Send + Sync>> {
        // Fetch order
        let order_row = sqlx::query!(
            r#"
            SELECT id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let Some(order_row) = order_row else {
            return Ok(None);
        };
        
        // Fetch order items
        let item_rows = sqlx::query!(
            r#"
            SELECT id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, metadata
            FROM order_items
            WHERE order_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let items = item_rows
            .into_iter()
            .map(|row| altis_order::OrderItem {
                id: row.id,
                product_id: row.product_id,
                product_type: row.product_type,
                product_code: row.product_code,
                name: row.name,
                description: row.description,
                price_nuc: row.price_nuc,
                quantity: row.quantity,
                status: row.status.parse().unwrap_or(altis_order::OrderItemStatus::Active),
                metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
            })
            .collect();
        
        let order = altis_order::Order {
            id: order_row.id,
            customer_id: order_row.customer_id,
            customer_email: order_row.customer_email,
            offer_id: order_row.offer_id,
            airline_id: order_row.airline_id,
            items,
            status: order_row.status.parse().unwrap_or(altis_order::OrderStatus::Proposed),
            total_nuc: order_row.total_nuc,
            currency: order_row.currency,
            payment_method: order_row.payment_method,
            payment_reference: order_row.payment_reference,
            created_at: order_row.created_at,
            updated_at: order_row.updated_at,
        };
        
        Ok(Some(order))
    }
    
    async fn update_order_status(
        &self,
        id: Uuid,
        status: altis_order::OrderStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query!(
            r#"
            UPDATE orders
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            status.to_string(),
            id
        )
        .execute(&self.pool)
        .await?;
        
        // Log status change
        sqlx::query!(
            r#"
            INSERT INTO order_changes (order_id, change_type, new_value)
            VALUES ($1, 'STATUS_CHANGE', $2)
            "#,
            id,
            serde_json::json!({"status": status.to_string()})
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: &altis_order::OrderItem,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = serde_json::to_value(&item.metadata)?;
        
        sqlx::query!(
            r#"
            INSERT INTO order_items (id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            item.id,
            order_id,
            item.product_id,
            item.product_type,
            item.product_code.as_deref(),
            item.name,
            item.description.as_deref(),
            item.price_nuc,
            item.quantity,
            item.status.to_string(),
            metadata
        )
        .execute(&self.pool)
        .await?;
        
        Ok(item.id)
    }
    
    async fn list_orders(
        &self,
        customer_id: &str,
    ) -> Result<Vec<altis_order::Order>, Box<dyn std::error::Error + Send + Sync>> {
        let order_rows = sqlx::query!(
            r#"
            SELECT id FROM orders
            WHERE customer_id = $1
            ORDER BY created_at DESC
            "#,
            customer_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut orders = Vec::new();
        for row in order_rows {
            if let Some(order) = self.get_order(row.id).await? {
                orders.push(order);
            }
        }
        
        Ok(orders)
    }
}
