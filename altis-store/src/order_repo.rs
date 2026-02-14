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
    customer_did: Option<String>,
    contact_phone: Option<String>,
    contact_first_name: Option<String>,
    contact_last_name: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct TravelerRow {
    id: Uuid,
    traveler_index: i32,
    ptc: String,
    first_name: String,
    last_name: String,
    date_of_birth: Option<chrono::NaiveDate>,
    gender: Option<String>,
    traveler_did: Option<String>,
    metadata: Option<Value>,
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
    revenue_status: Option<String>,
    operating_carrier_id: Option<Uuid>,
    net_rate_nuc: Option<i32>,
    commission_nuc: Option<i32>,
    metadata: Option<Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct LedgerRow {
    id: Uuid,
    order_id: Uuid,
    order_item_id: Uuid,
    transaction_type: String,
    amount_nuc: i32,
    currency: Option<String>,
    description: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
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

        let customer_id = order["customer_id"].as_str().ok_or(String::from("Missing customer id"))?;
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
        let customer_did = order["customer_did"].as_str();

        let contact_phone = order["contact_phone"].as_str();
        let contact_first_name = order["contact_first_name"].as_str();
        let contact_last_name = order["contact_last_name"].as_str();
        let expires_at_str = order["expires_at"].as_str();
        let expires_at = expires_at_str.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc)));

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO orders (id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference, customer_did, contact_phone, contact_first_name, contact_last_name, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(order_id)
        .bind(customer_id)
        .bind(customer_email)
        .bind(offer_id)
        .bind(airline_id)
        .bind(status)
        .bind(total_nuc)
        .bind(currency)
        .bind(payment_method)
        .bind(payment_reference)
        .bind(customer_did)
        .bind(contact_phone)
        .bind(contact_first_name)
        .bind(contact_last_name)
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;

        if let Some(travelers) = order["travelers"].as_array() {
            for traveler in travelers {
                let traveler_index = traveler["traveler_index"].as_i64().unwrap_or(0) as i32;
                let ptc = traveler["ptc"].as_str().unwrap_or("ADT");
                let first_name = traveler["first_name"].as_str().unwrap_or("Unknown");
                let last_name = traveler["last_name"].as_str().unwrap_or("Unknown");
                let dob_str = traveler["date_of_birth"].as_str();
                let dob = dob_str.and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
                let gender = traveler["gender"].as_str();
                let traveler_did = traveler["traveler_did"].as_str();
                let metadata = &traveler["metadata"];

                sqlx::query(
                    r#"
                    INSERT INTO travelers (order_id, traveler_index, ptc, first_name, last_name, date_of_birth, gender, traveler_did, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                .bind(order_id)
                .bind(traveler_index)
                .bind(ptc)
                .bind(first_name)
                .bind(last_name)
                .bind(dob)
                .bind(gender)
                .bind(traveler_did)
                .bind(metadata)
                .execute(&mut *tx)
                .await?;
            }
        }

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
                let operating_carrier_id_str = item["operating_carrier_id"].as_str();
                let operating_carrier_id = if let Some(id) = operating_carrier_id_str { Some(Uuid::parse_str(id)?) } else { None };
                let net_rate_nuc = item["net_rate_nuc"].as_i64().map(|v| v as i32);
                let commission_nuc = item["commission_nuc"].as_i64().map(|v| v as i32);
                let metadata = &item["metadata"];

                sqlx::query(
                    r#"
                    INSERT INTO order_items (id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, revenue_status, operating_carrier_id, net_rate_nuc, commission_nuc, metadata)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item_id)
                .bind(order_id)
                .bind(product_id)
                .bind(product_type)
                .bind(product_code)
                .bind(name)
                .bind(description)
                .bind(price_nuc)
                .bind(quantity)
                .bind(item_status)
                .bind("UNEARNED")
                .bind(operating_carrier_id)
                .bind(net_rate_nuc)
                .bind(commission_nuc)
                .bind(metadata)
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
        let order_row = sqlx::query_as::<_, OrderRow>(
            "SELECT id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference, customer_did, contact_phone, contact_first_name, contact_last_name, expires_at, created_at, updated_at FROM orders WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = order_row {
            let items_rows = sqlx::query_as::<_, OrderItemRow>(
                "SELECT id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, revenue_status, operating_carrier_id, net_rate_nuc, commission_nuc, metadata, created_at, updated_at FROM order_items WHERE order_id = $1"
            )
            .bind(id)
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
                    "revenue_status": item.revenue_status,
                    "operating_carrier_id": item.operating_carrier_id,
                    "net_rate_nuc": item.net_rate_nuc,
                    "commission_nuc": item.commission_nuc,
                    "metadata": item.metadata,
                    "created_at": item.created_at.map(|t| t.to_rfc3339()),
                    "updated_at": item.updated_at.map(|t| t.to_rfc3339())
                })
            }).collect();

            let fulfillment_rows = sqlx::query_as::<_, FulfillmentRow>(
                "SELECT id, order_id, order_item_id, fulfillment_type, barcode, qr_code_data, delivery_method, delivered_at, created_at FROM fulfillment WHERE order_id = $1"
            )
            .bind(id)
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
                    "delivered_at": f.delivered_at.map(|t| t.to_rfc3339()),
                    "created_at": f.created_at.map(|t| t.to_rfc3339())
                })
            }).collect();

            let traveler_rows = sqlx::query_as::<_, TravelerRow>(
                "SELECT id, traveler_index, ptc, first_name, last_name, date_of_birth, gender, traveler_did, metadata FROM travelers WHERE order_id = $1"
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?;

            let travelers: Vec<Value> = traveler_rows.into_iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "traveler_index": t.traveler_index,
                    "ptc": t.ptc,
                    "first_name": t.first_name,
                    "last_name": t.last_name,
                    "date_of_birth": t.date_of_birth.map(|d| d.format("%Y-%m-%d").to_string()),
                    "gender": t.gender,
                    "traveler_did": t.traveler_did,
                    "metadata": t.metadata
                })
            }).collect();

            let order_json = serde_json::json!({
                "id": row.id,
                "customer_id": row.customer_id,
                "customer_email": row.customer_email,
                "contact_info": {
                    "email": row.customer_email,
                    "phone": row.contact_phone,
                    "first_name": row.contact_first_name,
                    "last_name": row.contact_last_name,
                },
                "offer_id": row.offer_id,
                "airline_id": row.airline_id,
                "status": row.status,
                "total_nuc": row.total_nuc,
                "currency": row.currency,
                "payment_method": row.payment_method,
                "payment_reference": row.payment_reference,
                "customer_did": row.customer_did,
                "expires_at": row.expires_at.map(|t| t.to_rfc3339()),
                "items": items,
                "travelers": travelers,
                "fulfillment": fulfillment,
                "created_at": row.created_at.map(|t| t.to_rfc3339()),
                "updated_at": row.updated_at.map(|t| t.to_rfc3339())
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
        sqlx::query(
            "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(status)
        .bind(id)
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
        let status = String::from("ACTIVE");
        let operating_carrier_id_str = item["operating_carrier_id"].as_str();
        let operating_carrier_id = if let Some(id) = operating_carrier_id_str { Some(Uuid::parse_str(id)?) } else { None };
        let net_rate_nuc = item["net_rate_nuc"].as_i64().map(|v| v as i32);
        let commission_nuc = item["commission_nuc"].as_i64().map(|v| v as i32);
        let metadata = &item["metadata"];

        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, product_id, product_type, product_code, name, description, price_nuc, quantity, status, revenue_status, operating_carrier_id, net_rate_nuc, commission_nuc, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(item_id)
        .bind(order_id)
        .bind(product_id)
        .bind(product_type)
        .bind(product_code)
        .bind(name)
        .bind(description)
        .bind(price_nuc)
        .bind(quantity)
        .bind(status)
        .bind("UNEARNED")
        .bind(operating_carrier_id)
        .bind(net_rate_nuc)
        .bind(commission_nuc)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        Ok(item_id)
    }

    async fn list_orders(
        &self,
        customer_id: &str,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT id FROM orders WHERE customer_id = $1 ORDER BY created_at DESC",
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            let id: Uuid = sqlx::Row::get(&row, "id");
            if let Some(order) = self.get_order(id).await? {
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
        
        sqlx::query(
            r#"
            INSERT INTO fulfillment (id, order_id, order_item_id, fulfillment_type, barcode)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(fulfillment_id)
        .bind(order_id)
        .bind(order_item_id)
        .bind(fulfillment_type)
        .bind(barcode)
        .execute(&self.pool)
        .await?;

        Ok(fulfillment_id)
    }

    async fn consume_fulfillment(
        &self,
        barcode: &str,
        location: &str,
    ) -> Result<(Uuid, Uuid), Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            r#"
            UPDATE fulfillment 
            SET consumed_at = NOW(), consumption_location = $2
            WHERE barcode = $1
            RETURNING order_id, order_item_id
            "#,
        )
        .bind(barcode)
        .bind(location)
        .fetch_one(&self.pool)
        .await?;

        Ok((
            sqlx::Row::get(&row, "order_id"),
            sqlx::Row::get(&row, "order_item_id"),
        ))
    }

    async fn add_order_change(
        &self,
        order_id: Uuid,
        change_type: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            INSERT INTO order_changes (order_id, change_type, old_value, new_value, changed_by, reason)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(order_id)
        .bind(change_type)
        .bind(old_value)
        .bind(new_value)
        .bind(changed_by)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_orders_by_flight(
        &self,
        flight_id: &str,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, OrderRow>(
            "SELECT id, customer_id, customer_email, offer_id, airline_id, status, total_nuc, currency, payment_method, payment_reference, created_at, updated_at FROM orders WHERE id IN (SELECT order_id FROM order_items WHERE metadata->>'flight_id' = $1)"
        )
        .bind(flight_id)
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

    async fn add_order_ledger_entry(
        &self,
        order_id: Uuid,
        order_item_id: Uuid,
        transaction_type: &str,
        amount_nuc: i32,
        description: Option<&str>,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let entry_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO order_ledger (id, order_id, order_item_id, transaction_type, amount_nuc, description)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(entry_id)
        .bind(order_id)
        .bind(order_item_id)
        .bind(transaction_type)
        .bind(amount_nuc)
        .bind(description)
        .execute(&self.pool)
        .await?;
        Ok(entry_id)
    }

    async fn update_item_revenue_status(
        &self,
        item_id: Uuid,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "UPDATE order_items SET revenue_status = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(status)
        .bind(item_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_order_ledger(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, LedgerRow>(
            "SELECT id, order_id, order_item_id, transaction_type, amount_nuc, currency, description, created_at FROM order_ledger WHERE order_id = $1 ORDER BY created_at"
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        let ledger = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.id,
                "order_id": row.order_id,
                "order_item_id": row.order_item_id,
                "transaction_type": row.transaction_type,
                "amount_nuc": row.amount_nuc,
                "currency": row.currency,
                "description": row.description,
                "created_at": row.created_at.as_ref().map(|t| t.to_rfc3339())
            })
        }).collect();

        Ok(ledger)
    }
}
