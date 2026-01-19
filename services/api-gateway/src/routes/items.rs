use axum::{
    extract::{Path, State},
    routing::get,
    Extension, Json, Router,
};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{CorrectionLog, PublishableItem, User},
    AppState,
};

/// Public router for publishable items (accessed via /api/v1/items/:slug)
/// This provides public access to published content
pub fn public_router() -> Router<AppState> {
    Router::new().route("/:slug", get(get_item))
}

/// GET /api/v1/items/:slug
/// Get a published item by slug (public access)
/// This is the main public endpoint for accessing published conviction reports
async fn get_item(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    user: Option<Extension<User>>,
) -> ApiResult<Json<serde_json::Value>> {
    // Query for published item with the slug
    let query = "SELECT * FROM publishable_item WHERE slug = $slug AND status = 'published'";

    let mut result = state
        .db
        .client
        .query(query)
        .bind(("slug", slug.clone()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch published item: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch published item"))
        })?;

    let items: Vec<PublishableItem> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse published item: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse published item"))
    })?;

    let item = items
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Item not found".to_string()))?;

    // Get correction history for transparency
    let correction_query = format!(
        "SELECT * FROM correction_log WHERE item_id = publishable_item:{} ORDER BY corrected_at DESC",
        item.id.as_ref().unwrap_or(&"unknown".to_string())
    );

    let mut correction_result = state
        .db
        .client
        .query(&correction_query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch corrections: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch corrections"))
        })?;

    let corrections: Vec<CorrectionLog> = correction_result.take(0).unwrap_or_default();

    let username = user
        .map(|Extension(u)| u.username)
        .unwrap_or_else(|| "anonymous".to_string());

    tracing::info!("User {} viewed published item '{}'", username, slug);

    // Return published item with its correction history
    Ok(Json(serde_json::json!({
        "item": item,
        "corrections": corrections,
        "slug": slug
    })))
}
