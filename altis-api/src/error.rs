use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Not found: {0}")]
    NotFoundError(String),
    #[error("Conflict: {0}")]
    ConflictError(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFoundError(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::ConflictError(msg) => (StatusCode::CONFLICT, msg),
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal Server Error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            },
            AppError::Anyhow(err) => {
                tracing::error!("Internal Server Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            },
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
