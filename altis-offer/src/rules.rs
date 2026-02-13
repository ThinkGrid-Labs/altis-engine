use serde::{Deserialize, Serialize};
use altis_catalog::ProductType;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferRule {
    pub id: Uuid,
    pub name: String,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub priority: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    UserSegment(String),
    Origin(String),
    Destination(String),
    MinPassengers(i32),
    PriceRange(i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Bundle(ProductType),
    Discount(ProductType, f64), // (ProductType, DiscountPercentage)
    AddMetadata(String, serde_json::Value),
}

pub struct RuleEngine {
    rules: Vec<OfferRule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<OfferRule>) -> Self {
        let mut rules = rules;
        rules.sort_by_key(|r| -r.priority);
        Self { rules }
    }

    pub fn evaluate_bundling(&self, context: &serde_json::Value) -> Vec<ProductType> {
        let mut products_to_bundle = Vec::new();
        
        for rule in &self.rules {
            if !rule.is_active { continue; }
            
            if self.matches(rule, context) {
                for action in &rule.actions {
                    if let RuleAction::Bundle(product_type) = action {
                        products_to_bundle.push(product_type.clone());
                    }
                }
            }
        }
        
        products_to_bundle
    }

    pub fn evaluate_discount(&self, product_type: &ProductType, context: &serde_json::Value) -> f64 {
        let mut max_discount: f64 = 0.0;
        
        for rule in &self.rules {
            if !rule.is_active { continue; }
            
            if self.matches(rule, context) {
                for action in &rule.actions {
                    if let RuleAction::Discount(pt, discount) = action {
                        if pt == product_type {
                            max_discount = max_discount.max(*discount);
                        }
                    }
                }
            }
        }
        
        max_discount
    }

    fn matches(&self, rule: &OfferRule, context: &serde_json::Value) -> bool {
        for condition in &rule.conditions {
            match condition {
                RuleCondition::UserSegment(segment) => {
                    if context["user_segment"].as_str() != Some(segment) {
                        return false;
                    }
                }
                RuleCondition::Origin(origin) => {
                    if context["search"]["origin"].as_str() != Some(origin) {
                        return false;
                    }
                }
                RuleCondition::Destination(dest) => {
                    if context["search"]["destination"].as_str() != Some(dest) {
                        return false;
                    }
                }
                RuleCondition::MinPassengers(min) => {
                    if context["search"]["passengers"].as_i64().unwrap_or(0) < *min as i64 {
                        return false;
                    }
                }
                RuleCondition::PriceRange(_min, _max) => {
                    // TODO: Implement price range check if needed in generator
                }
            }
        }
        true
    }
}

pub fn get_default_rules() -> Vec<OfferRule> {
    vec![
        OfferRule {
            id: Uuid::new_v4(),
            name: "Premium Segment Bundle".to_string(),
            priority: 100,
            is_active: true,
            conditions: vec![RuleCondition::UserSegment("premium".to_string())],
            actions: vec![
                RuleAction::Bundle(ProductType::Lounge),
                RuleAction::Bundle(ProductType::FastTrack),
                RuleAction::Discount(ProductType::Lounge, 0.2),
            ],
        },
        OfferRule {
            id: Uuid::new_v4(),
            name: "Corporate Standard".to_string(),
            priority: 90,
            is_active: true,
            conditions: vec![RuleCondition::UserSegment("corporate".to_string())],
            actions: vec![
                RuleAction::Bundle(ProductType::Bag),
                RuleAction::Bundle(ProductType::Seat),
                RuleAction::Discount(ProductType::Seat, 0.5),
            ],
        },
        OfferRule {
            id: Uuid::new_v4(),
            name: "Family Bundle".to_string(),
            priority: 80,
            is_active: true,
            conditions: vec![RuleCondition::MinPassengers(3)],
            actions: vec![
                RuleAction::Bundle(ProductType::Bag),
                RuleAction::Discount(ProductType::Bag, 0.25),
            ],
        },
    ]
}
