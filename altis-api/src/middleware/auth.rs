use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

// ============================================================================
// JWT Claims
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub airline_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub exp: usize,
}

// ============================================================================
// Customer Authentication Middleware
// ============================================================================

pub async fn customer_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract token from Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 2. Decode and validate JWT
    let token_data = decode::<CustomerClaims>(
        token,
        &DecodingKey::from_secret(state.auth.secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 3. Check role is CUSTOMER
    if token_data.claims.role != "CUSTOMER" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 4. Inject claims into request extensions
    req.extensions_mut().insert(token_data.claims);
    
    Ok(next.run(req).await)
}

// ============================================================================
// Admin Authentication Middleware
// ============================================================================

pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 2. Decode JWT
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(state.auth.secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 3. Check role is ADMIN or SUPER_ADMIN
    if token_data.claims.role != "ADMIN" && token_data.claims.role != "SUPER_ADMIN" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 4. Inject claims
    req.extensions_mut().insert(token_data.claims);
    
    Ok(next.run(req).await)
}

// ============================================================================
// Permission Check Helper
// ============================================================================

pub fn has_permission(claims: &AdminClaims, permission: &str) -> bool {
    claims.permissions.contains(&permission.to_string())
}
