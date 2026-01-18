use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            ApiError::Authentication(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            ApiError::Authorization(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            ApiError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            ApiError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            ApiError::Internal(ref e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::Conflict(ref msg) => (StatusCode::CONFLICT, msg.as_str()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
