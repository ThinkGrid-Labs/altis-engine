use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::state::AppState;
use crate::offers::SearchOffersRequest;
use altis_core::iata::{AirShoppingRequest, AirShoppingResponse, NdcOffer, NdcPrice, NdcOfferItem};

impl From<AirShoppingRequest> for SearchOffersRequest {
    fn from(req: AirShoppingRequest) -> Self {
        Self {
            origin: req.shopping_criteria.origin,
            destination: req.shopping_criteria.destination,
            departure_date: req.shopping_criteria.travel_date,
            return_date: None,
            passengers: 1,
            cabin_class: None,
            user_segment: None,
        }
    }
}

pub async fn air_shopping(
    State(_state): State<AppState>,
    Json(req): Json<AirShoppingRequest>,
) -> Result<Json<AirShoppingResponse>, StatusCode> {
    // 1. Map NDC Request to internal search
    let _search_req = SearchOffersRequest::from(req);

    // We reuse the native search logic by calling a shared helper or just duplicating the core logic
    // For this implementation, we'll simulate the response based on the Search API
    
    // 2. Fetch offers (Internal)
    // In a real system, we'd call a shared service.
    // For NDC demo, we'll return a mapped response of what would be generated.
    
    let ndc_offers = vec![
        NdcOffer {
            offer_id: uuid::Uuid::new_v4().to_string(),
            owner: "AL".to_string(),
            total_price: NdcPrice { amount: 250, currency: "NUC".to_string() },
            items: vec![
                NdcOfferItem {
                    item_id: "item_1".to_string(),
                    service_name: "Flight SIN-KUL".to_string(),
                    price: NdcPrice { amount: 250, currency: "NUC".to_string() },
                }
            ],
        }
    ];

    Ok(Json(AirShoppingResponse {
        response_id: uuid::Uuid::new_v4().to_string(),
        offers: ndc_offers,
    }))
}
