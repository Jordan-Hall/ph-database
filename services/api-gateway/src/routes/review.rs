use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{Report, ReportStatus, User},
    services::AuditService,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queue", get(get_review_queue))
        .route("/:id/assign", post(assign_review))
        .route("/:id/decision", patch(make_review_decision))
        .route("/:id", get(get_review_detail))
}

#[derive(Debug, Deserialize)]
struct ReviewQueueQuery {
    #[serde(default = "default_limit")]
    limit: i32,
    #[serde(default)]
    status: Option<String>,
}

fn default_limit() -> i32 {
    50
}

#[derive(Debug, Serialize)]
struct ReviewQueueResponse {
    reports: Vec<ReportWithPriority>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportWithPriority {
    #[serde(flatten)]
    report: Report,
    priority_score: f64,
}

#[derive(Debug, Deserialize, Validate)]
struct ReviewDecisionRequest {
    decision: ReviewDecision,
    #[validate(length(min = 10, max = 1000))]
    notes: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewDecision {
    Approve,
    Reject,
    NeedsMoreInfo,
}

/// Get review queue with priority scoring (reviewer/admin only)
async fn get_review_queue(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(params): Query<ReviewQueueQuery>,
) -> ApiResult<Json<ReviewQueueResponse>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    // Use SurrealDB function for review queue with priority scoring
    let query = if let Some(status) = params.status {
        format!(
            "SELECT *, fn::calculate_priority(harm_risk, created_at) AS priority_score FROM report WHERE status = '{}' ORDER BY priority_score DESC LIMIT {}",
            status, params.limit
        )
    } else {
        format!(
            "SELECT *, fn::calculate_priority(harm_risk, created_at) AS priority_score FROM report WHERE status = 'submitted' ORDER BY priority_score DESC LIMIT {}",
            params.limit
        )
    };

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch review queue: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch review queue"))
    })?;

    let reports: Vec<ReportWithPriority> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse review queue: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse review queue"))
    })?;

    let total = reports.len();

    tracing::info!(
        "Reviewer {} fetched review queue with {} items",
        user.username,
        total
    );

    Ok(Json(ReviewQueueResponse { reports, total }))
}

/// Assign a report to a reviewer (reviewer/admin only)
async fn assign_review(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(report_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Update report status to under_review
    let update_query = format!(
        "UPDATE report:{} SET status = 'under_review', updated_at = time::now() RETURN AFTER",
        report_id
    );

    let updated: Vec<Report> = state
        .db
        .client
        .query(&update_query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to assign review: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to assign review"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated report: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse updated report"))
        })?;

    let _report = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Report not found or access denied".to_string()))?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "report.assign_review".to_string(),
            "report".to_string(),
            report_id.clone(),
            serde_json::json!({
                "status": "under_review",
                "reviewer_id": user_id
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Reviewer {} assigned report {} to themselves",
        user.username,
        report_id
    );

    Ok(Json(serde_json::json!({
        "message": "Report assigned successfully",
        "report_id": report_id,
        "status": "under_review"
    })))
}

/// Make review decision on a report (reviewer/admin only)
async fn make_review_decision(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(report_id): Path<String>,
    Json(payload): Json<ReviewDecisionRequest>,
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

    // Map decision to report status
    let new_status = match payload.decision {
        ReviewDecision::Approve => ReportStatus::Approved,
        ReviewDecision::Reject => ReportStatus::Rejected,
        ReviewDecision::NeedsMoreInfo => ReportStatus::NeedsMoreInfo,
    };

    // Update report status
    let update_query = format!(
        "UPDATE report:{} SET status = $status, updated_at = time::now() RETURN AFTER",
        report_id
    );

    let updated: Vec<Report> = state
        .db
        .client
        .query(&update_query)
        .bind(("status", serde_json::to_value(&new_status).unwrap()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update report status: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update report status"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated report: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse updated report"))
        })?;

    let _report = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Report not found or access denied".to_string()))?;

    // Create audit log
    let audit_service = AuditService::new(state.db.clone());
    audit_service
        .log(
            &user,
            "report.review_decision".to_string(),
            "report".to_string(),
            report_id.clone(),
            serde_json::json!({
                "decision": payload.decision,
                "new_status": new_status,
                "notes": payload.notes,
                "reviewer_id": user_id
            }),
            None,
            None,
        )
        .await?;

    tracing::info!(
        "Reviewer {} made decision {:?} on report {}",
        user.username,
        payload.decision,
        report_id
    );

    Ok(Json(serde_json::json!({
        "message": "Review decision recorded successfully",
        "report_id": report_id,
        "status": new_status,
        "decision": payload.decision
    })))
}

/// Get detailed report for review (reviewer/admin only)
async fn get_review_detail(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(report_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify reviewer or admin role
    if !user.roles.contains(&"reviewer".to_string())
        && !user.roles.contains(&"admin".to_string())
    {
        return Err(ApiError::Authorization(
            "Reviewer or admin role required".to_string(),
        ));
    }

    // Fetch report with priority score
    let query = format!(
        "SELECT *, fn::calculate_priority(harm_risk, created_at) AS priority_score FROM report WHERE id = report:{}",
        report_id
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch report: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch report"))
    })?;

    let reports: Vec<ReportWithPriority> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse report: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse report"))
    })?;

    let report = reports
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Report not found or access denied".to_string()))?;

    // Fetch evidence for this report
    let evidence_query = format!(
        "SELECT * FROM evidence WHERE report_id = report:{} ORDER BY created_at DESC",
        report_id
    );

    let mut evidence_result = state
        .db
        .client
        .query(&evidence_query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch evidence: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch evidence"))
        })?;

    let evidence: Vec<serde_json::Value> = evidence_result.take(0).map_err(|e| {
        tracing::error!("Failed to parse evidence: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse evidence"))
    })?;

    // Fetch audit logs for this report
    let audit_service = AuditService::new(state.db.clone());
    let audit_logs = audit_service
        .get_resource_logs("report", &report_id, 20)
        .await?;

    Ok(Json(serde_json::json!({
        "report": report,
        "evidence": evidence,
        "audit_trail": audit_logs,
        "evidence_count": evidence.len()
    })))
}
