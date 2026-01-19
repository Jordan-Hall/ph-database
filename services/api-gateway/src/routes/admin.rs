use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{AuditLog, User, UserStatus},
    services::AuditService,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id/roles", patch(update_user_roles))
        .route("/users/:id/status", patch(update_user_status))
        .route("/users/:id", delete(delete_user))
        .route("/audit-logs", get(list_audit_logs))
        .route("/audit-logs/:resource_type/:resource_id", get(get_resource_audit_logs))
        .route("/tenants", get(list_tenants))
}

#[derive(Debug, Deserialize)]
struct ListUsersQuery {
    #[serde(default)]
    status: Option<String>,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_per_page")]
    per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Serialize)]
struct UsersListResponse {
    users: Vec<User>,
    total: usize,
    page: u32,
    per_page: u32,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    user: User,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateRolesRequest {
    #[validate(length(min = 1))]
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateStatusRequest {
    status: UserStatus,
}

/// List all users (admin only)
async fn list_users(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<UsersListResponse>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Build query
    let query = if let Some(status) = params.status {
        format!(
            "SELECT * FROM user WHERE status = '{}' ORDER BY created_at DESC",
            status
        )
    } else {
        "SELECT * FROM user ORDER BY created_at DESC".to_string()
    };

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch users: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch users"))
    })?;

    let users: Vec<User> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse users: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse users"))
    })?;

    let total = users.len();

    tracing::info!("Admin {} listed {} users", admin.username, total);

    Ok(Json(UsersListResponse {
        users,
        total,
        page: params.page,
        per_page: params.per_page,
    }))
}

/// Get user by ID (admin/reviewer only)
async fn get_user(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<UserResponse>> {
    // Verify admin or reviewer role
    if !requester.roles.contains(&"admin".to_string())
        && !requester.roles.contains(&"reviewer".to_string())
    {
        return Err(ApiError::Authorization(
            "Admin or reviewer role required".to_string(),
        ));
    }

    let users: Vec<User> = state
        .db
        .select(&format!("user:{}", user_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch user"))
        })?;

    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse { user }))
}

/// Update user roles (admin only)
async fn update_user_roles(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateRolesRequest>,
) -> ApiResult<Json<UserResponse>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Validate roles
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let valid_roles = ["user", "reviewer", "admin", "business"];
    for role in &payload.roles {
        if !valid_roles.contains(&role.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid role: {}. Valid roles: user, reviewer, admin, business",
                role
            )));
        }
    }

    // Update roles
    let update_query = format!(
        "UPDATE user:{} SET roles = $roles, updated_at = time::now() RETURN AFTER",
        user_id
    );

    let updated: Vec<User> = state
        .db
        .client
        .query(&update_query)
        .bind(("roles", serde_json::to_value(&payload.roles).unwrap()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user roles: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update user roles"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated user: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse updated user"))
        })?;

    let user = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    tracing::info!(
        "Admin {} updated roles for user {} to {:?}",
        admin.username,
        user_id,
        payload.roles
    );

    Ok(Json(UserResponse { user }))
}

/// Update user status (admin only)
async fn update_user_status(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> ApiResult<Json<UserResponse>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Update status
    let update_query = format!(
        "UPDATE user:{} SET status = $status, updated_at = time::now() RETURN AFTER",
        user_id
    );

    let updated: Vec<User> = state
        .db
        .client
        .query(&update_query)
        .bind(("status", serde_json::to_value(&payload.status).unwrap()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user status: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update user status"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse updated user: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse updated user"))
        })?;

    let user = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    tracing::info!(
        "Admin {} updated status for user {} to {:?}",
        admin.username,
        user_id,
        payload.status
    );

    Ok(Json(UserResponse { user }))
}

/// Delete user (admin only)
async fn delete_user(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    // Instead of hard delete, we'll update status to "deleted"
    let update_query = format!(
        "UPDATE user:{} SET status = 'deleted', updated_at = time::now() RETURN AFTER",
        user_id
    );

    let updated: Vec<User> = state
        .db
        .client
        .query(&update_query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete user: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to delete user"))
        })?
        .take(0)
        .map_err(|e| {
            tracing::error!("Failed to parse deleted user: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse deleted user"))
        })?;

    let _user = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    tracing::info!("Admin {} deleted user {}", admin.username, user_id);

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully",
        "user_id": user_id
    })))
}

/// List audit logs with filters (admin only)
async fn list_audit_logs(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Query(params): Query<ListAuditLogsQuery>,
) -> ApiResult<Json<AuditLogsResponse>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    let audit_service = AuditService::new(state.db.clone());
    let logs = audit_service
        .get_all_logs(params.action, params.resource_type, params.limit)
        .await?;

    let total = logs.len();

    tracing::info!("Admin {} listed {} audit logs", admin.username, total);

    Ok(Json(AuditLogsResponse { logs, total }))
}

/// Get audit logs for a specific resource (admin only)
async fn get_resource_audit_logs(
    State(state): State<AppState>,
    Extension(admin): Extension<User>,
    Path((resource_type, resource_id)): Path<(String, String)>,
    Query(params): Query<ResourceAuditQuery>,
) -> ApiResult<Json<AuditLogsResponse>> {
    // Verify admin role
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Authorization(
            "Admin role required".to_string(),
        ));
    }

    let audit_service = AuditService::new(state.db.clone());
    let logs = audit_service
        .get_resource_logs(&resource_type, &resource_id, params.limit)
        .await?;

    let total = logs.len();

    tracing::info!(
        "Admin {} listed {} audit logs for {}:{}",
        admin.username,
        total,
        resource_type,
        resource_id
    );

    Ok(Json(AuditLogsResponse { logs, total }))
}

#[derive(Debug, Deserialize)]
struct ListAuditLogsQuery {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    resource_type: Option<String>,
    #[serde(default = "default_audit_limit")]
    limit: i32,
}

#[derive(Debug, Deserialize)]
struct ResourceAuditQuery {
    #[serde(default = "default_audit_limit")]
    limit: i32,
}

fn default_audit_limit() -> i32 {
    100
}

#[derive(Debug, Serialize)]
struct AuditLogsResponse {
    logs: Vec<AuditLog>,
    total: usize,
}

async fn list_tenants() -> &'static str {
    "TODO: Implement tenant management"
}
