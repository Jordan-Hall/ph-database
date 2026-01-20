use axum::{
    extract::{Path, State},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        AppealDecision, AppealStatus, CorrectionLog, CorrectionRequest, PublishRequest,
        PublishResponse, PublishStatus, PublishableItem, Report, ReportStatus,
        ReviewAppealRequest, SubmitAppealRequest, TakedownAppeal,
        TakedownRequest as TakedownRequestModel, User,
    },
    services::AuditService,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/report/:report_id", post(publish_report))
        .route("/item/:item_id/withdraw", patch(withdraw_item))
        .route("/item/:item_id/correct", post(add_correction))
        .route("/item/:item_id/takedown", post(request_takedown))
        .route("/item/:item_id/appeal", post(submit_appeal))
        .route("/appeals", get(list_appeals))
        .route("/appeals/:appeal_id", get(get_appeal))
        .route("/appeals/:appeal_id/review", post(review_appeal))
        .route("/item/:slug", get(get_published_item))
}

/// Publish an approved report as a public item (reviewer/admin only)
async fn publish_report(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(report_id): Path<String>,
    Json(payload): Json<PublishRequest>,
) -> ApiResult<Json<PublishResponse>> {
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

    // Verify report exists and is approved
    let reports: Vec<Report> = state
        .db
        .select(&format!("report:{}", report_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch report: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch report"))
        })?;

    let report = reports
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Report not found or access denied".to_string()))?;

    if report.status != ReportStatus::Approved {
        return Err(ApiError::Validation(
            "Only approved reports can be published".to_string(),
        ));
    }

    let now = Utc::now();

    // Create publishable item
    let item = PublishableItem {
        id: None,
        slug: payload.slug.clone(),
        title: payload.title,
        content_type: payload.content_type,
        content: payload.content,
        summary: payload.summary,
        source_report_id: Some(format!("report:{}", report_id)),
        status: PublishStatus::Published,
        visibility_tier: payload.visibility_tier,
        published_at: Some(now),
        published_by: Some(format!("user:{}", user_id)),
        corrections: vec![],
        takedown_reason: None,
        created_at: now,
        updated_at: now,
    };

    let created: Option<PublishableItem> =
        state.db.create("publishable_item", item).await.map_err(|e| {
            tracing::error!("Failed to create publishable item: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create publishable item"))
        })?;

    let item = created.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Publishable item created but not returned"))
    })?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "item.publish".to_string(),
            "publishable_item".to_string(),
            item.id.clone().unwrap_or_else(|| "unknown".to_string()),
            serde_json::json!({
                "source_report_id": report_id,
                "slug": payload.slug,
                "visibility_tier": item.visibility_tier
            }),
            None,
            None,
        )
        .await?;

    let public_url = format!("/public/{}", payload.slug);

    tracing::info!(
        "Reviewer {} published report {} as item with slug '{}'",
        user.username,
        report_id,
        payload.slug
    );

    Ok(Json(PublishResponse { item, public_url }))
}

/// Withdraw a published item (reviewer/admin only)
async fn withdraw_item(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<String>,
    Json(payload): Json<WithdrawRequest>,
) -> ApiResult<Json<serde_json::Value>> {
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

    // Update item status to withdrawn
    let update_query = format!(
        "UPDATE publishable_item:{} SET status = 'withdrawn', takedown_reason = $reason, updated_at = time::now() RETURN AFTER",
        item_id
    );

    let updated: Vec<PublishableItem> = state
        .db
        .client
        .query(&update_query)
        .bind(("reason", payload.reason.clone()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to withdraw item: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to withdraw item"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated item: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse updated item"))
        })?;

    let _item = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Item not found or access denied".to_string()))?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "item.withdraw".to_string(),
            "publishable_item".to_string(),
            item_id.clone(),
            serde_json::json!({
                "reason": payload.reason
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Reviewer {} withdrew item {}",
        user.username,
        item_id
    );

    Ok(Json(serde_json::json!({
        "message": "Item withdrawn successfully",
        "item_id": item_id,
        "status": "withdrawn"
    })))
}

/// Add a correction to a published item (reviewer/admin only)
async fn add_correction(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<String>,
    Json(payload): Json<CorrectionRequest>,
) -> ApiResult<Json<serde_json::Value>> {
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
    let now = Utc::now();

    // Create correction log
    let correction = CorrectionLog {
        id: None,
        item_id: format!("publishable_item:{}", item_id),
        correction_type: payload.correction_type.clone(),
        old_value: payload.old_value.clone(),
        new_value: payload.new_value.clone(),
        reason: payload.reason.clone(),
        corrected_by: format!("user:{}", user_id),
        corrected_at: now,
    };

    let created: Option<CorrectionLog> = state
        .db
        .create("correction_log", correction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create correction log: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create correction log"))
        })?;

    let correction = created.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Correction log created but not returned"))
    })?;

    // Update item's corrections array
    let update_query = format!(
        "UPDATE publishable_item:{} SET corrections += $correction, updated_at = time::now()",
        item_id
    );

    state
        .db
        .client
        .query(&update_query)
        .bind((
            "correction",
            serde_json::json!({
                "type": payload.correction_type,
                "reason": payload.reason,
                "corrected_at": now.to_rfc3339(),
                "corrected_by": user.username
            }),
        ))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update item corrections: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update item corrections"))
        })?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "item.correct".to_string(),
            "publishable_item".to_string(),
            item_id.clone(),
            serde_json::json!({
                "correction_type": payload.correction_type,
                "reason": payload.reason
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Reviewer {} added correction to item {}",
        user.username,
        item_id
    );

    Ok(Json(serde_json::json!({
        "message": "Correction added successfully",
        "correction": correction
    })))
}

/// Request takedown of a published item (any authenticated user)
async fn request_takedown(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<String>,
    Json(payload): Json<TakedownRequestModel>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());
    let now = Utc::now();

    // Create takedown request
    let takedown_req = serde_json::json!({
        "item_id": format!("publishable_item:{}", item_id),
        "requested_by": format!("user:{}", user_id),
        "requester_email": payload.requester_email,
        "reason": payload.reason,
        "evidence_description": payload.evidence_description,
        "status": "pending",
        "created_at": now
    });

    let created: Option<serde_json::Value> = state
        .db
        .create("takedown_request", takedown_req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create takedown request: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create takedown request"))
        })?;

    let _request = created.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Takedown request created but not returned"))
    })?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "item.request_takedown".to_string(),
            "publishable_item".to_string(),
            item_id.clone(),
            serde_json::json!({
                "reason": payload.reason
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "User {} requested takedown of item {}",
        user.username,
        item_id
    );

    Ok(Json(serde_json::json!({
        "message": "Takedown request submitted successfully",
        "item_id": item_id,
        "status": "pending"
    })))
}

/// Get a published item by slug (public access if published)
async fn get_published_item(
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

    // Get correction history
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

    let username = user.map(|Extension(u)| u.username).unwrap_or_else(|| "anonymous".to_string());
    tracing::info!("User {} viewed published item '{}'", username, slug);

    Ok(Json(serde_json::json!({
        "item": item,
        "corrections": corrections
    })))
}

#[derive(Debug, Deserialize, Validate)]
struct WithdrawRequest {
    #[validate(length(min = 10, max = 1000))]
    reason: String,
}

/// Submit an appeal for a takedown request (any authenticated user)
async fn submit_appeal(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<String>,
    Json(payload): Json<SubmitAppealRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());
    let now = Utc::now();

    // Check if takedown request exists
    let takedown_query = format!(
        "SELECT * FROM takedown_request WHERE id = {}",
        payload.takedown_request_id
    );

    let mut td_result = state.db.client.query(&takedown_query).await.map_err(|e| {
        tracing::error!("Failed to fetch takedown request: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch takedown request"))
    })?;

    let takedown_requests: Vec<serde_json::Value> = td_result.take(0).map_err(|e| {
        tracing::error!("Failed to parse takedown request: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse takedown request"))
    })?;

    if takedown_requests.is_empty() {
        return Err(ApiError::NotFound("Takedown request not found".to_string()));
    }

    // Create appeal
    let appeal = TakedownAppeal {
        id: None,
        takedown_request_id: payload.takedown_request_id.clone(),
        item_id: format!("publishable_item:{}", item_id),
        appellant_id: format!("user:{}", user_id),
        appellant_email: payload.appellant_email,
        appeal_reason: payload.appeal_reason.clone(),
        supporting_evidence: payload.supporting_evidence,
        status: AppealStatus::Pending,
        reviewed_by: None,
        review_notes: None,
        created_at: now,
        updated_at: now,
        reviewed_at: None,
    };

    let created: Option<TakedownAppeal> = state
        .db
        .create("takedown_appeal", appeal)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create appeal: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create appeal"))
        })?;

    let appeal = created.ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Appeal created but not returned"))
    })?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "appeal.submit".to_string(),
            "takedown_appeal".to_string(),
            appeal.id.clone().unwrap_or_else(|| "unknown".to_string()),
            serde_json::json!({
                "item_id": item_id,
                "takedown_request_id": payload.takedown_request_id
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Appeal submitted by user {} for item {}",
        user.username,
        item_id
    );

    Ok(Json(serde_json::json!({
        "message": "Appeal submitted successfully",
        "appeal": appeal,
        "status": "pending"
    })))
}

/// List all appeals (admin only)
async fn list_appeals(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify admin role
    if !user.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Fetch all appeals
    let query = "SELECT * FROM takedown_appeal ORDER BY created_at DESC";

    let mut result = state.db.client.query(query).await.map_err(|e| {
        tracing::error!("Failed to fetch appeals: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch appeals"))
    })?;

    let appeals: Vec<TakedownAppeal> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse appeals: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse appeals"))
    })?;

    let count = appeals.len();

    tracing::info!("Admin {} listed {} appeals", user.username, count);

    Ok(Json(serde_json::json!({
        "appeals": appeals,
        "count": count
    })))
}

/// Get a specific appeal (admin or appellant)
async fn get_appeal(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(appeal_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch appeal
    let appeals: Vec<TakedownAppeal> = state
        .db
        .select(&format!("takedown_appeal:{}", appeal_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch appeal: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch appeal"))
        })?;

    let appeal = appeals
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Appeal not found".to_string()))?;

    // Check access: admin or appellant
    let is_admin = user.roles.contains(&"admin".to_string());
    let is_appellant = appeal.appellant_id == format!("user:{}", user_id);

    if !is_admin && !is_appellant {
        return Err(ApiError::Authorization(
            "Access denied to this appeal".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "appeal": appeal
    })))
}

/// Review an appeal (admin only)
async fn review_appeal(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(appeal_id): Path<String>,
    Json(payload): Json<ReviewAppealRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify admin role
    if !user.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());
    let now = Utc::now();
    let review_notes = payload.review_notes.clone();

    // Map decision to status
    let new_status = match payload.decision {
        AppealDecision::Approve => AppealStatus::Approved,
        AppealDecision::Reject => AppealStatus::Rejected,
    };

    // Update appeal
    let update_query = format!(
        "UPDATE takedown_appeal:{} SET
         status = $status,
         reviewed_by = $reviewed_by,
         review_notes = $review_notes,
         reviewed_at = $reviewed_at,
         updated_at = $updated_at
         RETURN AFTER",
        appeal_id
    );

    let updated: Vec<TakedownAppeal> = state
        .db
        .client
        .query(&update_query)
        .bind(("status", serde_json::to_value(&new_status).unwrap()))
        .bind(("reviewed_by", format!("user:{}", user_id)))
        .bind(("review_notes", review_notes))
        .bind(("reviewed_at", now))
        .bind(("updated_at", now))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update appeal: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update appeal"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated appeal: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse appeal"))
        })?;

    let appeal = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Appeal not found".to_string()))?;

    // If approved, restore the item
    if new_status == AppealStatus::Approved {
        let item_id = appeal.item_id.replace("publishable_item:", "");
        let restore_query = format!(
            "UPDATE {} SET status = 'published', takedown_reason = NULL, updated_at = time::now()",
            appeal.item_id
        );

        state.db.client.query(&restore_query).await.ok();

        tracing::info!("Item {} restored after appeal approval", item_id);
    }

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "appeal.review".to_string(),
            "takedown_appeal".to_string(),
            appeal_id.clone(),
            serde_json::json!({
                "decision": payload.decision,
                "status": new_status,
                "reviewed_by": user.username
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Appeal {} reviewed by admin {}: {:?}",
        appeal_id,
        user.username,
        payload.decision
    );

    Ok(Json(serde_json::json!({
        "message": "Appeal reviewed successfully",
        "appeal": appeal,
        "decision": payload.decision
    })))
}
