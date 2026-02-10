use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::{state::AppState, error::AppError};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/guest", post(login_guest))
}

async fn login_guest(State(state): State<AppState>) -> Result<Json<AuthResponse>, AppError> {
    let my_claims = Claims {
        sub: format!("guest-{}", Uuid::new_v4()),
        role: "guest".to_owned(),
        exp: (Utc::now() + Duration::seconds(state.auth.expiration as i64)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(state.auth.secret.as_bytes()))
        .map_err(|e| AppError::InternalServerError(format!("Token encoding failed: {}", e)))?;

    Ok(Json(AuthResponse { token }))
}
