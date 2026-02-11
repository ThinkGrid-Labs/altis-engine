use crate::models::Offer;

/// AI-driven offer ranking (initial rule-based implementation)
pub struct OfferRanker {
    config: RankingConfig,
}

#[derive(Debug, Clone)]
pub struct RankingConfig {
    /// Weight for conversion probability (0.0 - 1.0)
    pub conversion_weight: f64,
    
    /// Weight for profit margin (0.0 - 1.0)
    pub margin_weight: f64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            conversion_weight: 0.6,
            margin_weight: 0.4,
        }
    }
}

impl OfferRanker {
    pub fn new(config: RankingConfig) -> Self {
        Self { config }
    }
    
    /// Rank offers by conversion probability × profit margin
    pub fn rank_offers(&self, offers: &mut Vec<Offer>) {
        offers.sort_by(|a, b| {
            let score_a = self.calculate_score(a);
            let score_b = self.calculate_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    /// Calculate ranking score for an offer
    fn calculate_score(&self, offer: &Offer) -> f64 {
        let conversion_score = self.estimate_conversion_probability(offer);
        let margin_score = self.calculate_margin_score(offer);
        
        (conversion_score * self.config.conversion_weight) +
        (margin_score * self.config.margin_weight)
    }
    
    /// Estimate conversion probability (rule-based for now)
    fn estimate_conversion_probability(&self, offer: &Offer) -> f64 {
        // Simple heuristic: fewer items = higher conversion
        // (customers prefer simplicity)
        let item_count = offer.items.len() as f64;
        
        if item_count == 1.0 {
            0.8 // Flight-only: high conversion
        } else if item_count <= 3.0 {
            0.6 // Small bundle: medium conversion
        } else {
            0.4 // Large bundle: lower conversion
        }
    }
    
    /// Calculate profit margin score
    fn calculate_margin_score(&self, offer: &Offer) -> f64 {
        // Normalize total price to 0-1 scale
        // Higher price = higher margin (simplified)
        let normalized_price = (offer.total_nuc as f64 / 100000.0).min(1.0);
        normalized_price
    }
    
    /// Future: ML model integration stub
    #[allow(dead_code)]
    async fn predict_conversion_ml(&self, _offer: &Offer) -> Result<f64, String> {
        // TODO: Integrate with ML model service
        // - Send offer features to gRPC/REST endpoint
        // - Receive conversion probability prediction
        Err("ML model not yet implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OfferItem;
    use uuid::Uuid;
    
    #[test]
    fn test_offer_ranking() {
        let mut ranker = OfferRanker::new(RankingConfig::default());
        
        let mut offers = vec![
            create_test_offer(1, 10000), // Flight-only, low price
            create_test_offer(3, 25000), // Bundle, high price
            create_test_offer(1, 30000), // Flight-only, high price
        ];
        
        ranker.rank_offers(&mut offers);
        
        // Flight-only with high price should rank highest
        assert_eq!(offers[0].items.len(), 1);
        assert_eq!(offers[0].total_nuc, 30000);
    }
    
    fn create_test_offer(item_count: usize, total_price: i32) -> Offer {
        let mut offer = Offer::new(None, serde_json::json!({}));
        offer.total_nuc = total_price;
        
        for _ in 0..item_count {
            offer.items.push(OfferItem::new(
                offer.id,
                "FLIGHT".to_string(),
                Uuid::new_v4(),
                "Test Product".to_string(),
                total_price / item_count as i32,
                serde_json::json!({}),
            ));
        }
        
        offer
    }
}
