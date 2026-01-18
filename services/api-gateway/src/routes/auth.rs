use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use validator::Validate;

use crate::{
    auth::AuthService,
    error::{ApiError, ApiResult},
    models::{AuthResponse, LoginRequest, RegisterRequest},
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

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.jwt_expiry_minutes,
        state.config.refresh_token_expiry_days,
    );

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

    let auth_service = AuthService::new(
        state.db.clone(),
        state.config.jwt_secret.clone(),
        state.config.jwt_expiry_minutes,
        state.config.refresh_token_expiry_days,
    );

    let response = auth_service.login(payload).await?;
    Ok(Json(response))
}

async fn refresh(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement refresh token logic
    Ok(Json(serde_json::json!({
        "message": "Token refresh not yet implemented"
    })))
}

async fn logout(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement logout logic (invalidate tokens)
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
