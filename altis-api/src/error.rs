use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    AuthenticationError(String),
    AuthorizationError(String),
    validation_error(String),
    NotFoundError(String),
    ConflictError(String),
    InternalServerError(String),
    Anyhow(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::validation_error(msg) => (StatusCode::BAD_REQUEST, msg),
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

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Anyhow(err.into())
    }
}
