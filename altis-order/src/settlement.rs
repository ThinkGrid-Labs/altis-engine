use crate::models::{Order, SettlementEvent, LedgerEntry};
use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;
use serde_json::{json, Value};

#[async_trait]
pub trait SettlementAdaptor {
    /// Convert an order and its ledger entries into a standard settlement format
    async fn adapt(&self, order: &Order, ledger: Vec<LedgerEntry>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct IataSwoAdaptor;

#[async_trait]
impl SettlementAdaptor for IataSwoAdaptor {
    async fn adapt(&self, order: &Order, ledger: Vec<LedgerEntry>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation of IATA Settlement with Orders (SwO) JSON-LD
        let mut events = Vec::new();

        for entry in ledger {
            let event = json!({
                "@type": "SettlementEvent",
                "id": entry.id,
                "orderReference": {
                    "id": order.id,
                    "externalID": order.payment_reference,
                },
                "financialDetails": {
                    "totalAmount": {
                        "value": entry.amount_nuc,
                        "currency": entry.currency,
                    },
                    "transactionType": entry.transaction_type,
                },
                "reportingDate": Utc::now().to_rfc3339(),
            });
            events.push(event);
        }

        Ok(json!({
            "@context": "https://iata.org/swo/v1",
            "settlementData": {
                "airlineId": order.airline_id,
                "events": events,
            }
        }))
    }
}

pub struct LegacyHotAdaptor;

#[async_trait]
impl SettlementAdaptor for LegacyHotAdaptor {
    async fn adapt(&self, order: &Order, ledger: Vec<LedgerEntry>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation of Legacy RET/HOT mapping logic
        // This maps internal high-level objects to legacy IATA transaction codes
        let mut items = Vec::new();

        for entry in ledger {
            items.push(json!({
                "trans_type": match entry.transaction_type.as_str() {
                    "REVENUE_RECOGNITION" => "TKTT", // Mocking legacy RET codes
                    "REFUND" => "RFND",
                    _ => "MISC",
                },
                "doc_number": format!("999-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("000")),
                "amount": entry.amount_nuc,
                "currency": entry.currency,
                "order_id": order.id,
                "timestamp": entry.created_at.to_rfc3339(),
            }));
        }

        Ok(json!(items))
    }
}
