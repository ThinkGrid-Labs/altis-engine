use crate::models::Offer;
use crate::features::{SearchContext, OfferFeatures};
use crate::events::OfferTelemetry;
use altis_shared::models::events::OfferGeneratedEvent;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod ranking {
    tonic::include_proto!("ranking");
}

use ranking::ranking_service_client::RankingServiceClient;
use ranking::{PredictConversionRequest, UserContext, SearchContext as ProtoSearchContext, OfferFeatures as ProtoOfferFeatures};

/// AI-driven offer ranking (initial rule-based implementation)
pub struct OfferRanker {
    config: altis_store::app_config::RankingConfig,
    telemetry: Option<Arc<OfferTelemetry>>,
    ml_client: Option<RankingServiceClient<Channel>>,
}

// Redundant local config removed, using altis_store::app_config::RankingConfig

impl OfferRanker {
    pub fn new(config: altis_store::app_config::RankingConfig, telemetry: Option<Arc<OfferTelemetry>>, ml_client: Option<RankingServiceClient<Channel>>) -> Self {
        Self { config, telemetry, ml_client }
    }
    
    /// Rank offers for a specific request
    pub async fn rank_offers_with_context(&mut self, search_context: &SearchContext, offers: &mut Vec<Offer>) {
        // 1. Assign experiment
        let use_ml = self.should_use_ml();
        let experiment_id = if use_ml { "ML_RANKER_V1" } else { "CONTROL" };

        for offer in offers.iter_mut() {
            // 2. Extract features
            let features = OfferFeatures::extract(search_context, offer);
            
            // 3. Calculate score (Rules or ML)
            let score = if use_ml {
                self.get_ml_score(search_context, offer, &features).await.unwrap_or_else(|_| self.calculate_rule_score(offer))
            } else {
                self.calculate_rule_score(offer)
            };
            
            // 4. Update metadata for tracking
            offer.metadata["experiment_id"] = serde_json::json!(experiment_id);
            offer.metadata["score"] = serde_json::json!(score);

            // 5. Log telemetry
            if let Some(ref tel) = self.telemetry {
                let event = OfferGeneratedEvent {
                    offer_id: offer.id,
                    customer_id: None, // TODO: Pull from context
                    timestamp: chrono::Utc::now().timestamp(),
                    search_context: serde_json::to_value(search_context).unwrap_or_default(),
                    features: serde_json::json!({
                        "days_until_departure": features.days_until_departure,
                        "is_weekend": features.is_weekend,
                        "hour_of_day": features.hour_of_day,
                        "is_domestic": features.is_domestic,
                        "passenger_count": features.passenger_count,
                        "price_per_passenger": features.price_per_passenger,
                        "item_count": features.item_count,
                    }),
                };
                let _ = tel.log_offer_generated(event).await;
            }
        }

        // 6. Sort
        offers.sort_by(|a, b| {
            let score_a = a.metadata["score"].as_f64().unwrap_or(0.0);
            let score_b = b.metadata["score"].as_f64().unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn should_use_ml(&self) -> bool {
        if self.config.ml_experiment_percentage <= 0.0 { return false; }
        if self.config.ml_experiment_percentage >= 1.0 { return true; }
        
        // Simple random assignment for illustration
        use rand::Rng;
        rand::thread_rng().gen_bool(self.config.ml_experiment_percentage)
    }

    async fn get_ml_score(&mut self, context: &SearchContext, offer: &Offer, _features: &OfferFeatures) -> Result<f64, String> {
        let client = self.ml_client.as_mut().ok_or("ML client not configured")?;
        
        let request = tonic::Request::new(PredictConversionRequest {
            user_context: Some(UserContext {
                user_id: "".to_string(), // TODO
                is_guest: true,
                session_id: "".to_string(),
            }),
            search_context: Some(ProtoSearchContext {
                origin: context.origin.clone(),
                destination: context.destination.clone(),
                departure_date: context.departure_date.clone(),
                passengers: context.passengers,
                cabin_class: context.cabin_class.clone().unwrap_or_default(),
                user_segment: context.user_segment.clone().unwrap_or_default(),
            }),
            offer_features: Some(ProtoOfferFeatures {
                offer_id: offer.id.to_string(),
                total_price_nuc: offer.total_nuc,
                product_codes: offer.items.iter().filter_map(|i| i.product_code.clone()).collect(),
                discount_percentage: 0.0, // TODO
            }),
        });

        let response = client.predict_conversion(request).await
            .map_err(|e| e.to_string())?;
            
        Ok(response.into_inner().probability)
    }
    
    /// Rank offers using rule-based scoring (deprecated but used as fallback/control)
    pub fn rank_offers(&self, offers: &mut Vec<Offer>) {
        offers.sort_by(|a, b| {
            let score_a = self.calculate_rule_score(a);
            let score_b = self.calculate_rule_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    /// Calculate ranking score for an offer
    fn calculate_rule_score(&self, offer: &Offer) -> f64 {
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
        // Higher score for offers with higher margin percentage
        // Average margin percentage across all items
        let mut total_margin = 0.0;
        let item_count = offer.items.len();
        
        if item_count == 0 { return 0.0; }

        for item in &offer.items {
            // Try to get margin_percentage from metadata (product repository should have populated this)
            let margin = item.metadata["margin_percentage"].as_f64().unwrap_or(0.15); // Default 15%
            total_margin += margin;
        }

        let avg_margin = total_margin / item_count as f64;
        
        // Combine average margin % with total price to prioritize high-value/high-margin bundles
        let normalized_price = (offer.total_nuc as f64 / 100000.0).min(1.0);
        
        (avg_margin * 0.7) + (normalized_price * 0.3)
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
    
    #[tokio::test]
    async fn test_offer_ranking() {
        let config = altis_store::app_config::RankingConfig {
            conversion_weight: 0.6,
            margin_weight: 0.4,
            ml_experiment_percentage: 0.0,
            ml_service_url: None,
        };
        let mut ranker = OfferRanker::new(config, None, None);
        
        let mut offers = vec![
            create_test_offer(1, 10000), // Flight-only, low price
            create_test_offer(3, 25000), // Bundle, high price
            create_test_offer(1, 30000), // Flight-only, high price
        ];
        
        let context = SearchContext {
            origin: "SFO".to_string(),
            destination: "LHR".to_string(),
            departure_date: "2024-12-01".to_string(),
            passengers: 1,
            cabin_class: None,
            user_segment: None,
        };

        ranker.rank_offers_with_context(&context, &mut offers).await;
        
        // Flight-only with high price should rank highest (due to rule-based fallback)
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
