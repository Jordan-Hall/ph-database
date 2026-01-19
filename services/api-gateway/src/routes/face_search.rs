use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    services::surrealdb_features::{FaceSearchResult, FaceSearchService},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(search_faces))
}

#[derive(Debug, Deserialize, Validate)]
struct FaceSearchRequest {
    #[validate(length(min = 100))]
    image_base64: String,
    purpose: Option<String>,
}

#[derive(Debug, Serialize)]
struct FaceSearchResponse {
    results: Vec<FaceSearchResult>,
    audit_id: String,
    message: String,
}

/// Search for faces using SurrealDB ML (ephemeral, privacy-preserving)
/// The query image is NEVER stored, only processed in-memory
async fn search_faces(
    State(state): State<AppState>,
    Json(payload): Json<FaceSearchRequest>,
) -> ApiResult<Json<FaceSearchResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // For now, use placeholder user ID
    // TODO: Extract from auth middleware when protected routes are configured
    let user_id = "user:placeholder".to_string();

    // Use SurrealDB ML for face search
    let face_service = FaceSearchService::new(state.db.clone());
    let results = face_service
        .search_faces(payload.image_base64, user_id.clone())
        .await?;

    tracing::info!(
        "Face search completed for user {} with {} results",
        user_id,
        results.len()
    );

    Ok(Json(FaceSearchResponse {
        results,
        audit_id: "logged".to_string(),
        message: "Search completed. Query image was not stored (ephemeral processing).".to_string(),
    }))
}
