use axum::{
    extract::State,
    http::HeaderMap,
    routing::post,
    Extension, Json, Router,
};
use validator::Validate;

use crate::{
    auth::AuthService,
    error::{ApiError, ApiResult},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserInfo},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate request
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Use SurrealDB native authentication
    let auth_service = AuthService::new(state.db.clone(), state.config.database_url.clone());

    let response = auth_service.register(payload).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate request
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Use SurrealDB native authentication
    let auth_service = AuthService::new(state.db.clone(), state.config.database_url.clone());

    let response = auth_service.login(payload).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AuthResponse>> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            ApiError::Authentication("Missing or invalid Authorization header".to_string())
        })?;

    // Verify the token using AuthService
    let auth_service = AuthService::new(state.db.clone(), state.config.database_url.clone());
    let user = auth_service.verify_token(token).await?;

    // SurrealDB manages token refresh internally, so we return the same token
    // In production, you might want to generate a new token here
    Ok(Json(AuthResponse {
        access_token: token.to_string(),
        refresh_token: token.to_string(),
        user: UserInfo {
            id: user.id.unwrap_or_default(),
            username: user.username,
            email: user.email,
            roles: user.roles,
        },
    }))
}

async fn logout(
    State(_state): State<AppState>,
    user: Option<Extension<User>>,
) -> ApiResult<Json<serde_json::Value>> {
    // With JWT tokens, logout is primarily client-side (client discards the token)
    // We verify the user is authenticated and log the action
    if let Some(Extension(user)) = user {
        tracing::info!("User {} logged out", user.username);
        Ok(Json(serde_json::json!({
            "message": "Logged out successfully",
            "username": user.username
        })))
    } else {
        // No authentication required for logout, but log it anyway
        tracing::info!("Anonymous logout attempt");
        Ok(Json(serde_json::json!({
            "message": "Logged out successfully"
        })))
    }
}
