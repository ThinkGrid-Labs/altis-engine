use crate::models::{Offer, OfferItem};
use altis_catalog::{Product, ProductType, PricingContext, PricingEngine};
use uuid::Uuid;

/// Offer generation strategies
pub enum OfferStrategy {
    /// Flight only (baseline)
    FlightOnly,
    /// Flight + comfort (seat, meal)
    Comfort,
    /// Flight + premium (lounge, fast-track, priority bag)
    Premium,
    /// Flight + eco-conscious (carbon offset)
    EcoFriendly,
    /// Custom bundle
    Custom(Vec<ProductType>),
}

/// Generates offers from search criteria
pub struct OfferGenerator {
    pricing_engine: PricingEngine,
}

impl OfferGenerator {
    pub fn new(pricing_engine: PricingEngine) -> Self {
        Self { pricing_engine }
    }
    
    /// Generate multiple offer variants for a search
    pub async fn generate_offers(
        &self,
        customer_id: Option<String>,
        search_context: serde_json::Value,
        flight_products: Vec<Product>,
        ancillary_products: Vec<Product>,
    ) -> Result<Vec<Offer>, OfferError> {
        let mut offers = Vec::new();
        
        // Strategy 1: Flight Only
        if let Some(offer) = self.create_offer(
            customer_id.clone(),
            search_context.clone(),
            &flight_products,
            &[],
            OfferStrategy::FlightOnly,
        ).await? {
            offers.push(offer);
        }
        
        // Strategy 2: Comfort Bundle
        if let Some(offer) = self.create_offer(
            customer_id.clone(),
            search_context.clone(),
            &flight_products,
            &ancillary_products,
            OfferStrategy::Comfort,
        ).await? {
            offers.push(offer);
        }
        
        // Strategy 3: Premium Bundle
        if let Some(offer) = self.create_offer(
            customer_id.clone(),
            search_context.clone(),
            &flight_products,
            &ancillary_products,
            OfferStrategy::Premium,
        ).await? {
            offers.push(offer);
        }
        
        Ok(offers)
    }
    
    /// Create a single offer based on strategy
    async fn create_offer(
        &self,
        customer_id: Option<String>,
        search_context: serde_json::Value,
        flight_products: &[Product],
        ancillary_products: &[Product],
        strategy: OfferStrategy,
    ) -> Result<Option<Offer>, OfferError> {
        let mut offer = Offer::new(customer_id, search_context);
        
        // Add flight products
        for flight in flight_products {
            let price = self.pricing_engine.apply_continuous_adjustment(
                flight.base_price_nuc,
                1.0,
            );
            
            let item = OfferItem::new(
                offer.id,
                format!("{:?}", flight.product_type),
                flight.id,
                flight.name.clone(),
                price,
                flight.metadata.clone(),
            );
            
            offer.add_item(item);
        }
        
        // Add ancillaries based on strategy
        match strategy {
            OfferStrategy::FlightOnly => {
                // No ancillaries
            },
            OfferStrategy::Comfort => {
                // Add seat + meal
                self.add_ancillaries(&mut offer, ancillary_products, &[
                    ProductType::Seat,
                    ProductType::Meal,
                ]);
            },
            OfferStrategy::Premium => {
                // Add lounge + fast-track + bag
                self.add_ancillaries(&mut offer, ancillary_products, &[
                    ProductType::Lounge,
                    ProductType::FastTrack,
                    ProductType::Bag,
                ]);
            },
            OfferStrategy::EcoFriendly => {
                // Add carbon offset
                self.add_ancillaries(&mut offer, ancillary_products, &[
                    ProductType::CarbonOffset,
                ]);
            },
            OfferStrategy::Custom(types) => {
                self.add_ancillaries(&mut offer, ancillary_products, &types);
            },
        }
        
        Ok(Some(offer))
    }
    
    /// Helper to add ancillary products
    fn add_ancillaries(
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
                    offer.id,
                    format!("{:?}", product.product_type),
                    product.id,
                    product.name.clone(),
                    price,
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
