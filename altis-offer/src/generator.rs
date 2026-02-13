use crate::models::{Offer, OfferItem};
use crate::rules::{RuleEngine, get_default_rules};
use altis_catalog::{Product, ProductType, PricingEngine, PricingContext};

/// Offer generation strategies
/// Offer generation strategies (Dynamic variants)
pub enum OfferStrategy {
    /// Baseline flight
    Baseline,
    /// Rule-based dynamic offer
    Dynamic,
    /// Personalized bundle
    Personalized,
}

/// Generates offers from search criteria
pub struct OfferGenerator {
    pricing_engine: PricingEngine,
    rule_engine: RuleEngine,
}

impl OfferGenerator {
    pub fn new(pricing_engine: PricingEngine) -> Self {
        Self { 
            pricing_engine,
            rule_engine: RuleEngine::new(get_default_rules()),
        }
    }
    
    /// Generate multiple offer variants for a search
    pub async fn generate_offers(
        &self,
        customer_id: Option<String>,
        user_segment: Option<String>,
        search_context: serde_json::Value,
        flight_products: Vec<Product>,
        ancillary_products: Vec<Product>,
    ) -> Result<Vec<Offer>, OfferError> {
        let mut offers = Vec::new();
        
        let mut context = search_context.clone();
        context["user_segment"] = serde_json::json!(user_segment);
        
        // Strategy 1: Baseline
        if let Some(offer) = self.create_offer(
            customer_id.clone(),
            user_segment.clone(),
            context.clone(),
            &flight_products,
            &ancillary_products,
            OfferStrategy::Baseline,
        ).await? {
            offers.push(offer);
        }
        
        // Strategy 2: Dynamic (Rule-based)
        if let Some(offer) = self.create_offer(
            customer_id.clone(),
            user_segment.clone(),
            context.clone(),
            &flight_products,
            &ancillary_products,
            OfferStrategy::Dynamic,
        ).await? {
            offers.push(offer);
        }
        
        Ok(offers)
    }
    
    /// Create a single offer based on strategy
    async fn create_offer(
        &self,
        customer_id: Option<String>,
        user_segment: Option<String>,
        context: serde_json::Value,
        flight_products: &[Product],
        ancillary_products: &[Product],
        strategy: OfferStrategy,
    ) -> Result<Option<Offer>, OfferError> {
        let mut offer = Offer::new(customer_id, None, context.clone());
        
        let pricing_context = PricingContext {
            user_segment,
            ..Default::default()
        };

        // Add flight products
        for flight in flight_products {
            let price = self.pricing_engine.apply_continuous_adjustment(
                flight.base_price_nuc,
                &pricing_context,
            );
            
            // Enrich metadata with flight details if missing
            let mut metadata = if flight.metadata.is_null() {
                serde_json::json!({})
            } else {
                flight.metadata.clone()
            };

            if let Some(obj) = metadata.as_object_mut() {
                if !obj.contains_key("origin") {
                    obj.insert("origin".to_string(), context["origin"].clone());
                }
                if !obj.contains_key("destination") {
                    obj.insert("destination".to_string(), context["destination"].clone());
                }
                if !obj.contains_key("departure_date") {
                    obj.insert("departure_date".to_string(), context["departure_date"].clone());
                }
                
                // Add flight times if available in context or already in metadata
                if !obj.contains_key("departure_time") && context["departure_time"].is_string() {
                    obj.insert("departure_time".to_string(), context["departure_time"].clone());
                }
                if !obj.contains_key("arrival_time") && context["arrival_time"].is_string() {
                    obj.insert("arrival_time".to_string(), context["arrival_time"].clone());
                }
            }

            let item = OfferItem::new(
                format!("{:?}", flight.product_type),
                Some(flight.id),
                None,
                flight.name.clone(),
                flight.description.clone(),
                price,
                1,
                metadata,
            );
            
            offer.add_item(item);
        }
        
        // Add ancillaries based on strategy
        match strategy {
            OfferStrategy::Baseline => {
                // No ancillaries for baseline
            },
            OfferStrategy::Dynamic => {
                // Evaluate rules for bundling
                let bundled_types = self.rule_engine.evaluate_bundling(&context);
                for pt in bundled_types {
                    if let Some(product) = ancillary_products.iter().find(|p| p.product_type == pt) {
                        let discount = self.rule_engine.evaluate_discount(&pt, &context);
                        let final_price = (product.base_price_nuc as f64 * (1.0 - discount)) as i32;
                        
                        let item = OfferItem::new(
                            format!("{:?}", pt),
                            Some(product.id),
                            None,
                            product.name.clone(),
                            product.description.clone(),
                            final_price,
                            1,
                            product.metadata.clone(),
                        );
                        offer.add_item(item);
                    }
                }
            },
            OfferStrategy::Personalized => {
                // TODO: Add complex personalization logic
            }
        }
        
        Ok(Some(offer))
    }
    
    /// Helper to add ancillary products
    pub fn _add_ancillaries(
        &self,
        offer: &mut Offer,
        ancillary_products: &[Product],
        product_types: &[ProductType],
    ) {
        for product_type in product_types {
            if let Some(product) = ancillary_products.iter()
                .find(|p| &p.product_type == product_type && p.is_active)
            {
                // Apply 10% bundle discount
                let price = (product.base_price_nuc as f64 * 0.9) as i32;
                
                let item = OfferItem::new(
                    format!("{:?}", product.product_type),
                    Some(product.id),
                    None, // product_code
                    product.name.clone(),
                    product.description.clone(),
                    price,
                    1, // quantity
                    product.metadata.clone(),
                );
                
                offer.add_item(item);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OfferError {
    #[error("No products available for offer generation")]
    NoProducts,
    
    #[error("Pricing calculation failed: {0}")]
    PricingFailed(String),
    
    #[error("Invalid search context: {0}")]
    InvalidContext(String),
}
