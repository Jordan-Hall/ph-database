use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, "InvalidInput", msg),
            AppError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "StorageError", msg),
            AppError::ProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "ProcessingError", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NotFound", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalError", msg),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
