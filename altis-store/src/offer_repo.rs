use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use altis_core::repository::OfferRepository;

pub struct PostgresOfferRepository {
    pub pool: PgPool,
}

#[async_trait]
impl OfferRepository for PostgresOfferRepository {
    async fn save_offer(
        &self,
        offer: &altis_offer::Offer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Serialize search context to JSON
        let search_context = serde_json::to_value(&offer.search_context)?;
        
        // Insert offer
        sqlx::query!(
            r#"
            INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, currency, status, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            offer.id,
            offer.customer_id.as_deref(),
            offer.airline_id,
            search_context,
            offer.total_nuc,
            offer.currency,
            offer.status.to_string(),
            offer.expires_at
        )
        .execute(&self.pool)
        .await?;
        
        // Insert offer items
        for item in &offer.items {
            let metadata = serde_json::to_value(&item.metadata)?;
            
            sqlx::query!(
                r#"
                INSERT INTO offer_items (id, offer_id, product_id, product_type, product_code, name, description, price_nuc, quantity, metadata)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                item.id,
                offer.id,
                item.product_id,
                item.product_type,
                item.product_code.as_deref(),
                item.name,
                item.description.as_deref(),
                item.price_nuc,
                item.quantity,
                metadata
            )
            .execute(&self.pool)
            .await?;
        }
        
        Ok(())
    }
    
    async fn get_offer(
        &self,
        id: Uuid,
    ) -> Result<Option<altis_offer::Offer>, Box<dyn std::error::Error + Send + Sync>> {
        // Fetch offer
        let offer_row = sqlx::query!(
            r#"
            SELECT id, customer_id, airline_id, search_context, total_nuc, currency, status, expires_at, created_at
            FROM offers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let Some(offer_row) = offer_row else {
            return Ok(None);
        };
        
        // Fetch offer items
        let item_rows = sqlx::query!(
            r#"
            SELECT id, product_id, product_type, product_code, name, description, price_nuc, quantity, metadata
            FROM offer_items
            WHERE offer_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let items = item_rows
            .into_iter()
            .map(|row| altis_offer::OfferItem {
                id: row.id,
                product_id: row.product_id,
                product_type: row.product_type,
                product_code: row.product_code,
                name: row.name,
                description: row.description,
                price_nuc: row.price_nuc,
                quantity: row.quantity,
                metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
            })
            .collect();
        
        let offer = altis_offer::Offer {
            id: offer_row.id,
            customer_id: offer_row.customer_id,
            airline_id: offer_row.airline_id,
            search_context: serde_json::from_value(offer_row.search_context)?,
            items,
            total_nuc: offer_row.total_nuc,
            currency: offer_row.currency,
            status: offer_row.status.parse().unwrap_or(altis_offer::OfferStatus::Active),
            expires_at: offer_row.expires_at,
            created_at: offer_row.created_at,
        };
        
        Ok(Some(offer))
    }
    
    async fn list_active_offers(
        &self,
        customer_id: &str,
    ) -> Result<Vec<altis_offer::Offer>, Box<dyn std::error::Error + Send + Sync>> {
        let offer_rows = sqlx::query!(
            r#"
            SELECT id FROM offers
            WHERE customer_id = $1 AND status = 'ACTIVE' AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
            customer_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut offers = Vec::new();
        for row in offer_rows {
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
        sqlx::query!(
            r#"
            UPDATE offers
            SET status = 'EXPIRED'
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
