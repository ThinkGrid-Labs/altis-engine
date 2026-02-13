use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::state::AppState;
use altis_core::iata::{OneOrderResponse, OneOrder, OneOrderItem, NdcPrice};
use crate::orders::OrderResponse;

pub async fn order_retrieve(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OneOrderResponse>, StatusCode> {
    // 1. Fetch internal order
    let order_json = state.order_repo.get_order(order_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let internal_order: OrderResponse = serde_json::from_value(order_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Map to IATA ONE Order format
    let one_order = OneOrder {
        order_id: internal_order.id.to_string(),
        external_id: None,
        status: internal_order.status,
        total_amount: NdcPrice {
            amount: internal_order.total_nuc,
            currency: internal_order.currency,
        },
        order_items: internal_order.items.into_iter().map(|item| OneOrderItem {
            item_id: item.id.to_string(),
            product_name: item.name,
            status: item.status,
            price: NdcPrice {
                amount: item.price_nuc,
                currency: "NUC".to_string(), // Default currency
            },
        }).collect(),
    };

    Ok(Json(OneOrderResponse { order: one_order }))
}
