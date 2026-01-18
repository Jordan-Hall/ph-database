use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "api-gateway",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
