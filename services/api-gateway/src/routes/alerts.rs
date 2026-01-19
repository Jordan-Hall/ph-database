use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        AlertPriority, AlertQueryParams, AlertStatus, CreateAlertRequest, MissingPersonAlert,
        UpdateAlertStatusRequest, User,
    },
    services::AuditService,
    AppState,
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/active", get(get_active_alerts))
        .route("/:id", get(get_alert))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_alert))
        .route("/:id/status", patch(update_alert_status))
        .route("/:id/verify", post(verify_alert))
        .route("/:id/resolve", post(resolve_alert))
}

/// Get all active alerts (public access)
async fn get_active_alerts(
    State(state): State<AppState>,
    Query(params): Query<AlertQueryParams>,
) -> ApiResult<Json<Vec<MissingPersonAlert>>> {
    let limit = params.limit.min(100);

    // Query active and verified alerts
    let query = format!(
        "SELECT * FROM missing_person_alert
         WHERE status IN ['active', 'verified']
         AND active_until > time::now()
         ORDER BY priority DESC, created_at DESC
         LIMIT {}",
        limit
    );

    let alerts: Vec<MissingPersonAlert> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch active alerts: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch alerts"))
    })?;

    Ok(Json(alerts))
}

/// Get alert by ID (public for active/verified, restricted for drafts)
async fn get_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>,
    user: Option<Extension<User>>,
) -> ApiResult<Json<MissingPersonAlert>> {
    let alerts: Vec<MissingPersonAlert> = state
        .db
        .select(&format!("missing_person_alert:{}", alert_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch alert"))
        })?;

    let alert = alerts
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Alert not found or access denied".to_string()))?;

    // Check access: public can see active/verified, creator/reviewer can see drafts
    if alert.status != AlertStatus::Active && alert.status != AlertStatus::Verified {
        if let Some(Extension(user)) = user {
            let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());
            if alert.created_by != format!("user:{}", user_id)
                && !user.roles.contains(&"reviewer".to_string())
                && !user.roles.contains(&"admin".to_string())
            {
                return Err(ApiError::Authorization(
                    "Access denied to draft alert".to_string(),
                ));
            }
        } else {
            return Err(ApiError::Authorization(
                "Authentication required for draft alerts".to_string(),
            ));
        }
    }

    Ok(Json(alert))
}

/// Create a new missing person alert (authenticated users)
async fn create_alert(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateAlertRequest>,
) -> ApiResult<Json<MissingPersonAlert>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());
    let now = Utc::now();

    // Generate unique alert ID
    let alert_id = format!(
        "MPA-{}-{}",
        now.format("%Y%m%d"),
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    );

    // Calculate active_until (default 30 days)
    let active_days = payload.active_days.unwrap_or(30);
    let active_until = now + Duration::days(active_days as i64);

    // Create alert
    let alert = MissingPersonAlert {
        id: None,
        alert_id: alert_id.clone(),
        full_name: payload.full_name,
        age: payload.age,
        description: payload.description,
        last_seen_date: payload.last_seen_date,
        photo_url: payload.photo_url,
        contact_info: payload.contact_info,
        status: AlertStatus::Draft,  // Starts as draft, needs verification
        priority: payload.priority,
        active_until,
        created_by: format!("user:{}", user_id),
        created_at: now,
        updated_at: now,
    };

    // Insert into database
    let query = r#"
        CREATE missing_person_alert CONTENT {
            alert_id: $alert_id,
            full_name: $full_name,
            age: $age,
            description: $description,
            last_seen_date: $last_seen_date,
            last_seen_location: $last_seen_location,
            photo_url: $photo_url,
            contact_info: $contact_info,
            status: "draft",
            priority: $priority,
            geofence_radius_km: $geofence_radius_km,
            active_until: $active_until,
            created_by: $created_by,
            created_at: $created_at,
            updated_at: $updated_at
        }
    "#;

    let mut result = state
        .db
        .client
        .query(query)
        .bind((
            "alert_id",
            serde_json::to_value(&alert.alert_id).unwrap(),
        ))
        .bind((
            "full_name",
            serde_json::to_value(&alert.full_name).unwrap(),
        ))
        .bind(("age", serde_json::to_value(&alert.age).unwrap()))
        .bind((
            "description",
            serde_json::to_value(&alert.description).unwrap(),
        ))
        .bind((
            "last_seen_date",
            serde_json::to_value(&alert.last_seen_date).unwrap(),
        ))
        .bind((
            "last_seen_location",
            serde_json::to_value(&payload.last_seen_location).unwrap(),
        ))
        .bind((
            "photo_url",
            serde_json::to_value(&alert.photo_url).unwrap(),
        ))
        .bind((
            "contact_info",
            serde_json::to_value(&alert.contact_info).unwrap(),
        ))
        .bind((
            "priority",
            serde_json::to_value(&alert.priority).unwrap(),
        ))
        .bind((
            "geofence_radius_km",
            serde_json::to_value(&payload.geofence_radius_km).unwrap(),
        ))
        .bind((
            "active_until",
            serde_json::to_value(&alert.active_until).unwrap(),
        ))
        .bind((
            "created_by",
            serde_json::to_value(&alert.created_by).unwrap(),
        ))
        .bind((
            "created_at",
            serde_json::to_value(&alert.created_at).unwrap(),
        ))
        .bind((
            "updated_at",
            serde_json::to_value(&alert.updated_at).unwrap(),
        ))
        .await
        .map_err(|e| {
            tracing::error!("Failed to create alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create alert"))
        })?;

    let created_alert: Option<MissingPersonAlert> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse created alert: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse alert"))
    })?;

    let created_alert = created_alert.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Alert creation returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "alert_created",
        "missing_person_alert",
        created_alert.id.as_deref(),
        Some(serde_json::json!({
            "alert_id": alert_id,
            "full_name": alert.full_name,
            "priority": alert.priority,
        })),
        None,
    )
    .await
    .ok();

    tracing::info!(
        "Alert created: {} by user {}",
        alert_id,
        user_id
    );

    Ok(Json(created_alert))
}

/// Update alert status (creator or reviewer only)
async fn update_alert_status(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(alert_id): Path<String>,
    Json(payload): Json<UpdateAlertStatusRequest>,
) -> ApiResult<Json<MissingPersonAlert>> {
    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch current alert
    let alerts: Vec<MissingPersonAlert> = state
        .db
        .select(&format!("missing_person_alert:{}", alert_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch alert"))
        })?;

    let alert = alerts
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Alert not found".to_string()))?;

    // Check permissions: creator or reviewer/admin
    let is_creator = alert.created_by == format!("user:{}", user_id);
    let is_reviewer = user.roles.contains(&"reviewer".to_string())
        || user.roles.contains(&"admin".to_string());

    if !is_creator && !is_reviewer {
        return Err(ApiError::Authorization(
            "Only alert creator or reviewers can update status".to_string(),
        ));
    }

    // Update status
    let query = r#"
        UPDATE $alert_id SET
            status = $status,
            updated_at = time::now()
    "#;

    let mut result = state
        .db
        .client
        .query(query)
        .bind((
            "alert_id",
            serde_json::to_value(&format!("missing_person_alert:{}", alert_id)).unwrap(),
        ))
        .bind((
            "status",
            serde_json::to_value(&payload.status).unwrap(),
        ))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update alert status: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update status"))
        })?;

    let updated_alert: Option<MissingPersonAlert> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse updated alert: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse alert"))
    })?;

    let updated_alert = updated_alert.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Alert update returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "alert_status_updated",
        "missing_person_alert",
        Some(&alert_id),
        Some(serde_json::json!({
            "old_status": alert.status,
            "new_status": payload.status,
            "resolution_notes": payload.resolution_notes,
        })),
        None,
    )
    .await
    .ok();

    Ok(Json(updated_alert))
}

/// Verify an alert (reviewer/admin only) - makes it active
async fn verify_alert(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(alert_id): Path<String>,
) -> ApiResult<Json<MissingPersonAlert>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Update to active status
    let query = r#"
        UPDATE $alert_id SET
            status = "active",
            updated_at = time::now()
    "#;

    let mut result = state
        .db
        .client
        .query(query)
        .bind((
            "alert_id",
            serde_json::to_value(&format!("missing_person_alert:{}", alert_id)).unwrap(),
        ))
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to verify alert"))
        })?;

    let updated_alert: Option<MissingPersonAlert> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse verified alert: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse alert"))
    })?;

    let updated_alert = updated_alert.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Alert verification returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "alert_verified",
        "missing_person_alert",
        Some(&alert_id),
        Some(serde_json::json!({
            "verified_by": user_id,
        })),
        None,
    )
    .await
    .ok();

    tracing::info!("Alert {} verified by user {}", alert_id, user_id);

    Ok(Json(updated_alert))
}

/// Resolve an alert (creator or reviewer)
async fn resolve_alert(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(alert_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResult<Json<MissingPersonAlert>> {
    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch current alert
    let alerts: Vec<MissingPersonAlert> = state
        .db
        .select(&format!("missing_person_alert:{}", alert_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch alert"))
        })?;

    let alert = alerts
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Alert not found".to_string()))?;

    // Check permissions: creator or reviewer/admin
    let is_creator = alert.created_by == format!("user:{}", user_id);
    let is_reviewer = user.roles.contains(&"reviewer".to_string())
        || user.roles.contains(&"admin".to_string());

    if !is_creator && !is_reviewer {
        return Err(ApiError::Authorization(
            "Only alert creator or reviewers can resolve alerts".to_string(),
        ));
    }

    let resolution_notes = payload
        .get("resolution_notes")
        .and_then(|v| v.as_str())
        .unwrap_or("Alert resolved");

    // Update to resolved status
    let query = r#"
        UPDATE $alert_id SET
            status = "resolved",
            resolved_at = time::now(),
            updated_at = time::now()
    "#;

    let mut result = state
        .db
        .client
        .query(query)
        .bind((
            "alert_id",
            serde_json::to_value(&format!("missing_person_alert:{}", alert_id)).unwrap(),
        ))
        .await
        .map_err(|e| {
            tracing::error!("Failed to resolve alert: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to resolve alert"))
        })?;

    let updated_alert: Option<MissingPersonAlert> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse resolved alert: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse alert"))
    })?;

    let updated_alert = updated_alert.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Alert resolution returned no data"))
    })?;

    // Create audit log
    AuditService::log_action(
        &state.db,
        &user_id,
        "alert_resolved",
        "missing_person_alert",
        Some(&alert_id),
        Some(serde_json::json!({
            "resolved_by": user_id,
            "resolution_notes": resolution_notes,
        })),
        None,
    )
    .await
    .ok();

    tracing::info!("Alert {} resolved by user {}", alert_id, user_id);

    Ok(Json(updated_alert))
}
