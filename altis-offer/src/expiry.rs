use crate::models::{Offer, OfferStatus};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Manages offer expiry and cleanup
pub struct ExpiryManager {
    offers: HashMap<Uuid, Offer>,
}

impl ExpiryManager {
    pub fn new() -> Self {
        Self {
            offers: HashMap::new(),
        }
    }
    
    /// Store an offer with automatic expiry tracking
    pub fn store_offer(&mut self, offer: Offer) {
        self.offers.insert(offer.id, offer);
    }
    
    /// Get an offer if it's still active
    pub fn get_offer(&self, offer_id: &Uuid) -> Option<&Offer> {
        self.offers.get(offer_id).filter(|o| o.is_active())
    }
    
    /// Mark an offer as accepted
    pub fn accept_offer(&mut self, offer_id: &Uuid) -> Result<(), ExpiryError> {
        let offer = self.offers.get_mut(offer_id)
            .ok_or_else(|| ExpiryError::NotFound(offer_id.to_string()))?;
        
        if !offer.is_active() {
            return Err(ExpiryError::Expired(offer_id.to_string()));
        }
        
        offer.status = OfferStatus::Accepted;
        Ok(())
    }
    
    /// Clean up expired offers
    pub fn cleanup_expired(&mut self) -> usize {
        let now = Utc::now();
        let initial_count = self.offers.len();
        
        self.offers.retain(|_, offer| {
            if offer.expires_at <= now && offer.status == OfferStatus::Active {
                false // Remove expired active offers
            } else {
                true // Keep non-expired or already processed offers
            }
        });
        
        initial_count - self.offers.len()
    }
    
    /// Get count of active offers
    pub fn active_count(&self) -> usize {
        self.offers.values().filter(|o| o.is_active()).count()
    }
}

impl Default for ExpiryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpiryError {
    #[error("Offer not found: {0}")]
    NotFound(String),
    
    #[error("Offer expired: {0}")]
    Expired(String),
    
    #[error("Offer already processed: {0}")]
    AlreadyProcessed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    
    #[test]
    fn test_offer_expiry() {
        let mut manager = ExpiryManager::new();
        
        let mut offer = Offer::new(None, serde_json::json!({}));
        let offer_id = offer.id;
        
        // Store active offer
        manager.store_offer(offer.clone());
        assert!(manager.get_offer(&offer_id).is_some());
        
        // Manually expire the offer
        offer.expires_at = Utc::now() - Duration::minutes(1);
        manager.store_offer(offer);
        
        // Should not be retrievable
        assert!(manager.get_offer(&offer_id).is_none());
        
        // Cleanup should remove it
        let removed = manager.cleanup_expired();
        assert_eq!(removed, 1);
    }
}
