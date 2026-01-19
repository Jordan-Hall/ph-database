use axum::{
    extract::State,
    routing::post,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::User,
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
    Extension(user): Extension<User>,
    Json(payload): Json<FaceSearchRequest>,
) -> ApiResult<Json<FaceSearchResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check if user has permission to search faces (reviewer or admin only)
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Face search requires reviewer or admin role".to_string(),
        ));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Use SurrealDB ML for face search
    let face_service = FaceSearchService::new(state.db.clone());
    let results = face_service
        .search_faces(payload.image_base64, user_id.clone())
        .await?;

    tracing::info!(
        "Face search completed for user {} ({}) with {} results",
        user.username,
        user_id,
        results.len()
    );

    Ok(Json(FaceSearchResponse {
        results,
        audit_id: "logged".to_string(),
        message: "Search completed. Query image was not stored (ephemeral processing).".to_string(),
    }))
}
