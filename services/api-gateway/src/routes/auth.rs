use axum::{
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Extension, Json, Router,
};
use validator::Validate;
use totp_rs::{Algorithm, Secret, TOTP};
use chrono::Utc;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::{
    auth::AuthService,
    error::{ApiError, ApiResult},
    models::{
        AuthResponse, LoginRequest, RegisterRequest, User, UserInfo,
        MfaSecret, MfaSetupResponse, MfaEnableRequest, MfaVerifyRequest,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/enable", post(mfa_enable))
        .route("/mfa/disable", post(mfa_disable))
        .route("/mfa/verify", post(mfa_verify))
        .route("/mfa/backup-codes", get(mfa_regenerate_backup_codes))
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
        mfa_required: None,
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

// ============================================================================
// MULTI-FACTOR AUTHENTICATION (MFA)
// ============================================================================

/// Generate MFA secret and QR code for user (authenticated users only)
#[axum::debug_handler]
async fn mfa_setup(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> ApiResult<Json<MfaSetupResponse>> {
    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Check if user already has MFA setup
    let check_query = format!(
        "SELECT * FROM mfa_secret WHERE user_id = user:{}",
        user_id.replace("user:", "")
    );

    let mut result = state.db.client.query(&check_query).await.map_err(|e| {
        tracing::error!("Failed to check existing MFA: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to check MFA status"))
    })?;

    let existing: Vec<MfaSecret> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse MFA secrets: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse MFA secrets"))
    })?;

    // If MFA already exists and is verified, don't allow re-setup
    if !existing.is_empty() && user.mfa_enabled {
        return Err(ApiError::Validation(
            "MFA is already enabled. Disable it first to re-setup.".to_string()
        ));
    }

    // Generate new TOTP secret
    let secret = Secret::generate_secret();
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("PredatorHuntersDB".to_string()),
        user.email.clone(),
    ).map_err(|e| {
        tracing::error!("Failed to create TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create TOTP"))
    })?;

    // Generate QR code URL
    let qr_code_url = totp.get_qr_base64().map_err(|e| {
        tracing::error!("Failed to generate QR code: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to generate QR code"))
    })?;

    // Generate 10 backup codes using UUID
    let mut backup_codes = Vec::new();
    let mut hashed_backup_codes = Vec::new();

    for _ in 0..10 {
        // Use UUID v4 and take first 8 characters for backup code
        let code = uuid::Uuid::new_v4()
            .to_string()
            .replace("-", "")
            .chars()
            .take(8)
            .collect::<String>()
            .to_uppercase();

        let hashed = hash(&code, DEFAULT_COST).map_err(|e| {
            tracing::error!("Failed to hash backup code: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to hash backup code"))
        })?;

        backup_codes.push(code);
        hashed_backup_codes.push(hashed);
    }

    // Store MFA secret in database
    let mfa_secret = MfaSecret {
        id: None,
        user_id: format!("user:{}", user_id.replace("user:", "")),
        secret: secret.to_encoded().to_string(),
        backup_codes: hashed_backup_codes,
        created_at: Utc::now(),
        verified_at: None,
    };

    let created: Option<MfaSecret> = state.db.create("mfa_secret", mfa_secret).await.map_err(|e| {
        tracing::error!("Failed to store MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to store MFA secret"))
    })?;

    if created.is_none() {
        return Err(ApiError::Internal(anyhow::anyhow!("Failed to create MFA secret")));
    }

    tracing::info!("User {} initiated MFA setup", user.username);

    Ok(Json(MfaSetupResponse {
        secret: secret.to_encoded().to_string(),
        qr_code_url: format!("data:image/png;base64,{}", qr_code_url),
        backup_codes,
        manual_entry_key: secret.to_encoded().to_string(),
    }))
}

/// Enable MFA after verifying TOTP code (authenticated users only)
async fn mfa_enable(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<MfaEnableRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch MFA secret
    let query = format!(
        "SELECT * FROM mfa_secret WHERE user_id = user:{}",
        user_id.replace("user:", "")
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch MFA secret"))
    })?;

    let secrets: Vec<MfaSecret> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse MFA secrets: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse MFA secrets"))
    })?;

    let mfa_secret = secrets.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("MFA not setup. Please setup MFA first.".to_string())
    })?;

    // Verify TOTP code
    let secret = Secret::Encoded(mfa_secret.secret.clone()).to_bytes().map_err(|e| {
        tracing::error!("Failed to decode secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to decode secret"))
    })?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("PredatorHuntersDB".to_string()),
        user.email.clone(),
    ).map_err(|e| {
        tracing::error!("Failed to create TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create TOTP"))
    })?;

    let is_valid = totp.check_current(&payload.code).map_err(|e| {
        tracing::error!("Failed to verify TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to verify TOTP"))
    })?;

    if !is_valid {
        return Err(ApiError::Authentication("Invalid MFA code".to_string()));
    }

    // Enable MFA for user
    let update_user_query = format!(
        "UPDATE user:{} SET mfa_enabled = true, updated_at = time::now()",
        user_id.replace("user:", "")
    );

    state.db.client.query(&update_user_query).await.map_err(|e| {
        tracing::error!("Failed to enable MFA: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to enable MFA"))
    })?;

    // Mark MFA secret as verified
    let update_secret_query = format!(
        "UPDATE {} SET verified_at = time::now()",
        mfa_secret.id.unwrap_or_default()
    );

    state.db.client.query(&update_secret_query).await.map_err(|e| {
        tracing::error!("Failed to mark MFA as verified: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to mark MFA as verified"))
    })?;

    tracing::info!("User {} enabled MFA", user.username);

    Ok(Json(serde_json::json!({
        "message": "MFA enabled successfully",
        "mfa_enabled": true
    })))
}

/// Disable MFA (authenticated users only)
async fn mfa_disable(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<MfaVerifyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    if !user.mfa_enabled {
        return Err(ApiError::Validation("MFA is not enabled".to_string()));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch MFA secret
    let query = format!(
        "SELECT * FROM mfa_secret WHERE user_id = user:{}",
        user_id.replace("user:", "")
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch MFA secret"))
    })?;

    let secrets: Vec<MfaSecret> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse MFA secrets: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse MFA secrets"))
    })?;

    let mfa_secret = secrets.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("MFA secret not found".to_string())
    })?;

    // Verify TOTP code before disabling
    let secret = Secret::Encoded(mfa_secret.secret.clone()).to_bytes().map_err(|e| {
        tracing::error!("Failed to decode secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to decode secret"))
    })?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("PredatorHuntersDB".to_string()),
        user.email.clone(),
    ).map_err(|e| {
        tracing::error!("Failed to create TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create TOTP"))
    })?;

    let is_valid = totp.check_current(&payload.code).map_err(|e| {
        tracing::error!("Failed to verify TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to verify TOTP"))
    })?;

    if !is_valid {
        // Check if it's a backup code
        let mut is_backup_valid = false;
        for hashed_code in &mfa_secret.backup_codes {
            if verify(&payload.code, hashed_code).unwrap_or(false) {
                is_backup_valid = true;
                break;
            }
        }

        if !is_backup_valid {
            return Err(ApiError::Authentication("Invalid MFA code".to_string()));
        }
    }

    // Disable MFA for user
    let update_user_query = format!(
        "UPDATE user:{} SET mfa_enabled = false, updated_at = time::now()",
        user_id.replace("user:", "")
    );

    state.db.client.query(&update_user_query).await.map_err(|e| {
        tracing::error!("Failed to disable MFA: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to disable MFA"))
    })?;

    // Delete MFA secret
    let delete_query = format!(
        "DELETE {}",
        mfa_secret.id.unwrap_or_default()
    );

    state.db.client.query(&delete_query).await.map_err(|e| {
        tracing::error!("Failed to delete MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to delete MFA secret"))
    })?;

    tracing::info!("User {} disabled MFA", user.username);

    Ok(Json(serde_json::json!({
        "message": "MFA disabled successfully",
        "mfa_enabled": false
    })))
}

/// Verify TOTP code (for testing purposes, authenticated users only)
async fn mfa_verify(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<MfaVerifyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    if !user.mfa_enabled {
        return Err(ApiError::Validation("MFA is not enabled".to_string()));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch MFA secret
    let query = format!(
        "SELECT * FROM mfa_secret WHERE user_id = user:{}",
        user_id.replace("user:", "")
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch MFA secret"))
    })?;

    let secrets: Vec<MfaSecret> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse MFA secrets: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse MFA secrets"))
    })?;

    let mfa_secret = secrets.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("MFA secret not found".to_string())
    })?;

    // Verify TOTP code
    let secret = Secret::Encoded(mfa_secret.secret.clone()).to_bytes().map_err(|e| {
        tracing::error!("Failed to decode secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to decode secret"))
    })?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("PredatorHuntersDB".to_string()),
        user.email.clone(),
    ).map_err(|e| {
        tracing::error!("Failed to create TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create TOTP"))
    })?;

    let is_valid = totp.check_current(&payload.code).map_err(|e| {
        tracing::error!("Failed to verify TOTP: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to verify TOTP"))
    })?;

    if !is_valid {
        // Check if it's a backup code
        let mut is_backup_valid = false;
        for hashed_code in &mfa_secret.backup_codes {
            if verify(&payload.code, hashed_code).unwrap_or(false) {
                is_backup_valid = true;
                break;
            }
        }

        if !is_backup_valid {
            return Err(ApiError::Authentication("Invalid MFA code".to_string()));
        }

        tracing::info!("User {} verified MFA with backup code", user.username);
    } else {
        tracing::info!("User {} verified MFA with TOTP", user.username);
    }

    Ok(Json(serde_json::json!({
        "message": "MFA code verified successfully",
        "valid": true
    })))
}

/// Regenerate backup codes (authenticated users only)
async fn mfa_regenerate_backup_codes(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.mfa_enabled {
        return Err(ApiError::Validation("MFA is not enabled".to_string()));
    }

    let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

    // Fetch MFA secret
    let query = format!(
        "SELECT * FROM mfa_secret WHERE user_id = user:{}",
        user_id.replace("user:", "")
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to fetch MFA secret: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to fetch MFA secret"))
    })?;

    let secrets: Vec<MfaSecret> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse MFA secrets: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse MFA secrets"))
    })?;

    let mfa_secret = secrets.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("MFA secret not found".to_string())
    })?;

    // Generate 10 new backup codes using UUID
    let mut backup_codes = Vec::new();
    let mut hashed_backup_codes = Vec::new();

    for _ in 0..10 {
        // Use UUID v4 and take first 8 characters for backup code
        let code = uuid::Uuid::new_v4()
            .to_string()
            .replace("-", "")
            .chars()
            .take(8)
            .collect::<String>()
            .to_uppercase();

        let hashed = hash(&code, DEFAULT_COST).map_err(|e| {
            tracing::error!("Failed to hash backup code: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to hash backup code"))
        })?;

        backup_codes.push(code);
        hashed_backup_codes.push(hashed);
    }

    // Update MFA secret with new backup codes
    let update_query = format!(
        "UPDATE {} SET backup_codes = $codes, updated_at = time::now()",
        mfa_secret.id.unwrap_or_default()
    );

    state.db.client.query(&update_query)
        .bind(("codes", hashed_backup_codes))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update backup codes: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to update backup codes"))
        })?;

    tracing::info!("User {} regenerated backup codes", user.username);

    Ok(Json(serde_json::json!({
        "message": "Backup codes regenerated successfully",
        "backup_codes": backup_codes
    })))
}
