use axum::{
    extract::State,
    Json,

    routing::post,
    Router,
};
use serde::Serialize;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::{state::AppState, error::AppError, middleware::auth::CustomerClaims};

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/guest", post(login_guest))
}

async fn login_guest(State(state): State<AppState>) -> Result<Json<AuthResponse>, AppError> {
    let my_claims = CustomerClaims {
        sub: format!("guest-{}", Uuid::new_v4()),
        email: None,
        role: "GUEST".to_owned(),
        exp: (Utc::now() + Duration::seconds(state.auth.expiration as i64)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(state.auth.secret.as_bytes()))
        .map_err(|e| AppError::InternalServerError(format!("Token encoding failed: {}", e)))?;

    Ok(Json(AuthResponse { token }))
}
