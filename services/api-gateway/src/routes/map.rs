use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        CreateMapEntryRequest, DisplayPolicy, HarmRisk, MapBoundsQuery, MapEntry, PrecisionClass,
        User, VisibilityTier,
    },
    services::AuditService,
    AppState,
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/entries", get(get_map_entries))
        .route("/entries/:id", get(get_map_entry))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/entries", post(create_map_entry))
        .route("/entries/:id", patch(update_map_entry))
        .route("/entries/:id", delete(delete_map_entry))
        .route("/entries/:id/verify", post(verify_map_entry))
}

/// Get map entries within bounds (public access, respects RLAC)
async fn get_map_entries(
    State(state): State<AppState>,
    Query(params): Query<MapBoundsQuery>,
) -> ApiResult<Json<Vec<MapEntry>>> {
    let limit = params.limit.min(500);  // Max 500 entries per request

    // Build query with geo bounding box
    let query = format!(
        r#"SELECT * FROM map_entry
           WHERE geometry.coordinates[0] >= {}
           AND geometry.coordinates[0] <= {}
           AND geometry.coordinates[1] >= {}
           AND geometry.coordinates[1] <= {}
           LIMIT {}"#,
        params.west, params.east, params.south, params.north, limit
    );

    let entries: Vec<MapEntry> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch map entries: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch map entries"))
    })?;

    // Apply precision-based display policy
    let filtered_entries = entries
        .into_iter()
        .map(|mut entry| {
            // Fuzzy locations: reduce precision for public display
            if entry.display_policy == DisplayPolicy::Fuzzy {
                // Round coordinates to ~100m precision for fuzzy display
                if let Some(coords) = entry.geometry.get_mut("coordinates") {
                    if let Some(arr) = coords.as_array_mut() {
                        if arr.len() >= 2 {
                            if let Some(lon) = arr[0].as_f64() {
                                arr[0] = serde_json::json!((lon * 1000.0).round() / 1000.0);
                            }
                            if let Some(lat) = arr[1].as_f64() {
                                arr[1] = serde_json::json!((lat * 1000.0).round() / 1000.0);
                            }
                        }
                    }
                }
            }

            // Hidden entries should not appear in public results (handled by RLAC)
            entry
        })
        .collect();

    Ok(Json(filtered_entries))
}

/// Get a single map entry by ID (public access for public tier)
async fn get_map_entry(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
    user: Option<Extension<User>>,
) -> ApiResult<Json<MapEntry>> {
    let entries: Vec<MapEntry> = state
        .db
        .select(&format!("map_entry:{}", entry_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch map entry: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch map entry"))
        })?;

    let mut entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Map entry not found or access denied".to_string()))?;

    // Apply display policy if not reviewer
    let is_reviewer = user.as_ref().map_or(false, |Extension(u)| {
        u.roles.contains(&"reviewer".to_string()) || u.roles.contains(&"admin".to_string())
    });

    if !is_reviewer && entry.display_policy == DisplayPolicy::Fuzzy {
        // Apply fuzzy display
        if let Some(coords) = entry.geometry.get_mut("coordinates") {
            if let Some(arr) = coords.as_array_mut() {
                if arr.len() >= 2 {
                    if let Some(lon) = arr[0].as_f64() {
                        arr[0] = serde_json::json!((lon * 1000.0).round() / 1000.0);
                    }
                    if let Some(lat) = arr[1].as_f64() {
                        arr[1] = serde_json::json!((lat * 1000.0).round() / 1000.0);
                    }
                }
            }
        }
    }

    Ok(Json(entry))
}

/// Create a new map entry (reviewer/admin only)
async fn create_map_entry(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateMapEntryRequest>,
) -> ApiResult<Json<MapEntry>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Validate coordinates
    if payload.latitude < -90.0 || payload.latitude > 90.0 {
        return Err(ApiError::Validation(
            "Latitude must be between -90 and 90".to_string(),
        ));
    }
    if payload.longitude < -180.0 || payload.longitude > 180.0 {
        return Err(ApiError::Validation(
            "Longitude must be between -180 and 180".to_string(),
        ));
    }

    // Create GeoJSON Point
    let geometry = serde_json::json!({
        "type": "Point",
        "coordinates": [payload.longitude, payload.latitude]
    });

    // Create map entry
    let query = r#"
        CREATE map_entry CONTENT {
            geometry: $geometry,
            precision_class: $precision_class,
            display_policy: $display_policy,
            linked_item_id: $linked_item_id,
            linked_conviction_id: $linked_conviction_id,
            street_name: $street_name,
            city: $city,
            postcode_district: $postcode_district,
            visibility_tier: $visibility_tier,
            harm_risk: $harm_risk,
            verified: false,
            created_at: $created_at
        }
    "#;

    let mut result = state
        .db
        .client
        .query(query)
        .bind(("geometry", geometry))
        .bind((
            "precision_class",
            serde_json::to_value(&payload.precision_class).unwrap(),
        ))
        .bind((
            "display_policy",
            serde_json::to_value(&payload.display_policy).unwrap(),
        ))
        .bind((
            "linked_item_id",
            payload
                .linked_item_id
                .as_ref()
                .map(|id| format!("publishable_item:{}", id)),
        ))
        .bind((
            "linked_conviction_id",
            payload
                .linked_conviction_id
                .as_ref()
                .map(|id| format!("conviction_record:{}", id)),
        ))
        .bind(("street_name", payload.street_name.clone()))
        .bind(("city", payload.city.clone()))
        .bind(("postcode_district", payload.postcode_district.clone()))
        .bind((
            "visibility_tier",
            serde_json::to_value(&payload.visibility_tier).unwrap(),
        ))
        .bind((
            "harm_risk",
            serde_json::to_value(&payload.harm_risk).unwrap(),
        ))
        .bind(("created_at", Utc::now().to_rfc3339()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to create map entry: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create map entry"))
        })?;

    let created_entry: Option<MapEntry> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse created map entry: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse map entry"))
    })?;

    let created_entry = created_entry.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Map entry creation returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "map_entry_created",
        "map_entry",
        created_entry.id.as_deref(),
        Some(serde_json::json!({
            "location": format!("{}, {}", payload.city, payload.street_name),
            "precision_class": payload.precision_class,
            "harm_risk": payload.harm_risk,
        })),
        None,
    )
    .await
    .ok();

    tracing::info!("Map entry created by user {}", user_id);

    Ok(Json(created_entry))
}

/// Update a map entry (reviewer/admin only)
async fn update_map_entry(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(entry_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResult<Json<MapEntry>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch current entry
    let entries: Vec<MapEntry> = state
        .db
        .select(&format!("map_entry:{}", entry_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch map entry: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch map entry"))
        })?;

    let _current_entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Map entry not found".to_string()))?;

    // Build update query dynamically based on provided fields
    let mut updates = Vec::new();

    if let Some(precision_class) = payload.get("precision_class") {
        updates.push(format!("precision_class = {}", precision_class));
    }
    if let Some(display_policy) = payload.get("display_policy") {
        updates.push(format!("display_policy = {}", display_policy));
    }
    if let Some(visibility_tier) = payload.get("visibility_tier") {
        updates.push(format!("visibility_tier = {}", visibility_tier));
    }
    if let Some(harm_risk) = payload.get("harm_risk") {
        updates.push(format!("harm_risk = {}", harm_risk));
    }

    if updates.is_empty() {
        return Err(ApiError::Validation(
            "No valid fields to update".to_string(),
        ));
    }

    let query = format!(
        "UPDATE map_entry:{} SET {}",
        entry_id,
        updates.join(", ")
    );

    let mut result: Vec<MapEntry> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to update map entry: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to update map entry"))
    })?;

    let updated_entry: Option<MapEntry> = result.pop();

    let updated_entry = updated_entry.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Map entry update returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "map_entry_updated",
        "map_entry",
        Some(&entry_id),
        Some(payload),
        None,
    )
    .await
    .ok();

    Ok(Json(updated_entry))
}

/// Delete a map entry (admin only)
async fn delete_map_entry(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(entry_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify admin role
    if !user.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization("Admin role required".to_string()));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Delete entry
    let query = format!("DELETE map_entry:{}", entry_id);

    let _deleted: Vec<MapEntry> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to delete map entry: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to delete map entry"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "map_entry_deleted",
        "map_entry",
        Some(&entry_id),
        None,
        None,
    )
    .await
    .ok();

    tracing::info!("Map entry {} deleted by admin {}", entry_id, user_id);

    Ok(Json(serde_json::json!({
        "message": "Map entry deleted successfully",
        "id": entry_id
    })))
}

/// Verify a map entry (reviewer/admin only)
async fn verify_map_entry(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(entry_id): Path<String>,
) -> ApiResult<Json<MapEntry>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Update verification status
    let query = format!("UPDATE map_entry:{} SET verified = true", entry_id);

    let mut result: Vec<MapEntry> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to verify map entry: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to verify map entry"))
    })?;

    let verified_entry: Option<MapEntry> = result.pop();

    let verified_entry = verified_entry.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Map entry verification returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "map_entry_verified",
        "map_entry",
        Some(&entry_id),
        Some(serde_json::json!({
            "verified_by": user_id,
        })),
        None,
    )
    .await
    .ok();

    tracing::info!("Map entry {} verified by user {}", entry_id, user_id);

    Ok(Json(verified_entry))
}
