use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Context for pricing calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingContext {
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Is this product part of a bundle?
    pub is_bundled: bool,
    
    /// User segment (for personalized pricing)
    pub user_segment: Option<String>,
    
    /// Time-based multiplier (peak hours, etc.)
    pub time_multiplier: Option<f64>,
    
    /// Demand-based multiplier
    pub demand_multiplier: Option<f64>,
    
    /// Additional context metadata
    pub metadata: serde_json::Value,
}

impl Default for PricingContext {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            is_bundled: false,
            user_segment: None,
            time_multiplier: Some(1.0),
            demand_multiplier: Some(1.0),
            metadata: serde_json::json!({}),
        }
    }
}

/// Continuous pricing engine
pub struct PricingEngine {
    /// Base configuration
    config: PricingConfig,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    /// Minimum price adjustment (in cents)
    pub min_adjustment_cents: i32,
    
    /// Maximum multiplier allowed
    pub max_multiplier: f64,
    
    /// Minimum multiplier allowed
    pub min_multiplier: f64,
    
    /// Enable continuous pricing
    pub enable_continuous: bool,
    
    /// Multipliers for different user segments (e.g., "premium" => 1.2)
    pub segment_multipliers: HashMap<String, f64>,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            min_adjustment_cents: 1,
            max_multiplier: 3.0,
            min_multiplier: 0.5,
            enable_continuous: true,
            segment_multipliers: {
                let mut m = HashMap::new();
                m.insert("premium".to_string(), 1.2);
                m.insert("corporate".to_string(), 1.1);
                m.insert("economy".to_string(), 1.0);
                m.insert("leisure".to_string(), 0.95);
                m
            },
        }
    }
}

impl PricingEngine {
    pub fn new(config: PricingConfig) -> Self {
        Self { config }
    }
    
    /// Calculate continuous price adjustment based on demand
    pub fn calculate_demand_multiplier(&self, available_inventory: i32, total_capacity: i32) -> f64 {
        if total_capacity == 0 {
            return 1.0;
        }
        
        let utilization = 1.0 - (available_inventory as f64 / total_capacity as f64);
        
        // Exponential curve: as utilization increases, price increases
        let multiplier = 1.0 + (utilization * utilization * 2.0);
        
        // Clamp to configured limits
        multiplier.max(self.config.min_multiplier).min(self.config.max_multiplier)
    }
    
    /// Calculate time-based multiplier
    pub fn calculate_time_multiplier(&self, departure_time: DateTime<Utc>) -> f64 {
        let now = Utc::now();
        let hours_until_departure = (departure_time - now).num_hours();
        
        // Last-minute booking premium
        if hours_until_departure < 24 {
            1.5
        } else if hours_until_departure < 72 {
            1.2
        } else if hours_until_departure > 720 { // 30 days
            0.9 // Early bird discount
        } else {
            1.0
        }
    }
    
    /// Apply continuous pricing adjustment (by cents)
    pub fn apply_continuous_adjustment(&self, base_price: i32, context: &PricingContext) -> i32 {
        if !self.config.enable_continuous {
            return base_price;
        }
        
        // Start with demand multiplier if present in context, or 1.0
        let mut multiplier = context.demand_multiplier.unwrap_or(1.0);
        
        // Factor in time multiplier
        if let Some(tm) = context.time_multiplier {
            multiplier *= tm;
        }
        
        // Factor in segment multiplier
        if let Some(segment) = &context.user_segment {
            if let Some(sm) = self.config.segment_multipliers.get(segment) {
                multiplier *= sm;
            }
        }
        
        let adjusted = (base_price as f64 * multiplier) as i32;
        
        // Round to nearest cent
        let remainder = adjusted % self.config.min_adjustment_cents;
        if remainder >= self.config.min_adjustment_cents / 2 {
            adjusted + (self.config.min_adjustment_cents - remainder)
        } else {
            adjusted - remainder
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demand_multiplier() {
        let engine = PricingEngine::new(PricingConfig::default());
        
        // Low demand (90% available)
        let multiplier = engine.calculate_demand_multiplier(90, 100);
        assert!(multiplier < 1.1);
        
        // High demand (10% available)
        let multiplier = engine.calculate_demand_multiplier(10, 100);
        assert!(multiplier > 2.0);
    }
    
    #[test]
    fn test_continuous_adjustment() {
        let engine = PricingEngine::new(PricingConfig::default());
        
        let base_price = 10000; // $100.00
        let adjusted = engine.apply_continuous_adjustment(base_price, 1.234);
        
        // Should be rounded to nearest cent
        assert_eq!(adjusted, 12340);
    }
}
