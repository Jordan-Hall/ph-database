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
    models::{User, UserStatus},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id/roles", patch(update_user_roles))
        .route("/users/:id/status", patch(update_user_status))
        .route("/users/:id", delete(delete_user))
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

async fn list_tenants() -> &'static str {
    "TODO: Implement tenant management"
}
