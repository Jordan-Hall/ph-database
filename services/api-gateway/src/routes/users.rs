use axum::{
    extract::State,
    routing::{get, patch},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::User,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_current_user))
        .route("/me", patch(update_current_user))
}

#[derive(Debug, Serialize)]
struct UserProfileResponse {
    user: User,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 50))]
    username: Option<String>,
    #[validate(email)]
    email: Option<String>,
}

/// Get current user profile
async fn get_current_user(
    Extension(user): Extension<User>,
) -> ApiResult<Json<UserProfileResponse>> {
    Ok(Json(UserProfileResponse { user }))
}

/// Update current user profile
async fn update_current_user(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateProfileRequest>,
) -> ApiResult<Json<UserProfileResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user
        .id
        .clone()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("User ID not found")))?;

    // If no fields to update, return error
    if payload.username.is_none() && payload.email.is_none() {
        return Err(ApiError::Validation(
            "At least one field must be provided for update".to_string(),
        ));
    }

    // Build a JSON object with the updates
    let mut update_data = serde_json::json!({});

    if let Some(username) = payload.username {
        update_data["username"] = serde_json::Value::String(username);
    }

    if let Some(email) = payload.email {
        // Check if email is already taken by another user
        let check_query = format!(
            "SELECT * FROM user WHERE email = $email AND id != user:{}",
            user_id
        );
        let mut result = state
            .db
            .client
            .query(&check_query)
            .bind(("email", email.clone()))
            .await
            .map_err(|e| {
                tracing::error!("Failed to check email uniqueness: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to check email uniqueness"))
            })?;

        let existing: Vec<User> = result.take(0).map_err(|e| {
            tracing::error!("Failed to parse email check result: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse email check result"))
        })?;

        if !existing.is_empty() {
            return Err(ApiError::Validation(
                "Email already in use by another user".to_string(),
            ));
        }

        update_data["email"] = serde_json::Value::String(email);
    }

    update_data["updated_at"] = serde_json::json!("time::now()");

    // Perform update using MERGE
    let update_query = format!(
        "UPDATE user:{} MERGE $data RETURN AFTER",
        user_id
    );

    let mut query_builder = state.db.client.query(&update_query);
    query_builder = query_builder.bind(("data", update_data));

    let mut result = query_builder.await.map_err(|e| {
        tracing::error!("Failed to update user profile: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to update user profile"))
    })?;

    let updated: Vec<User> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse updated user: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse updated user"))
    })?;

    let user = updated
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    tracing::info!("User {} updated their profile", user.username);

    Ok(Json(UserProfileResponse { user }))
}
