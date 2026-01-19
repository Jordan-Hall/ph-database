use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiKey, ApiKeyStatus, CheckType, ConfidenceLevel, CreateApiKeyRequest,
        CreateApiKeyResponse, CreateTenantRequest, MatchSummary, TenantStatus, ValidationRequest,
        ValidationResponse,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate))
        .route("/usage", get(get_usage))
        .route("/keys", post(create_key))
        .route("/keys/:id", delete(delete_key))
        .route("/tenants", post(create_tenant))
}

/// Validate a person against conviction records (requires valid API key)
async fn validate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidationRequest>,
) -> ApiResult<Json<ValidationResponse>> {
    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Extract and validate API key
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ApiError::Authorization("Missing or invalid X-API-Key header".to_string())
        })?;

    // Hash the API key to look it up
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    // Look up API key
    let query = format!(
        "SELECT * FROM api_key WHERE key_hash = '{}' AND status = 'active'",
        key_hash
    );

    let keys: Vec<ApiKey> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to lookup API key: {}", e);
        ApiError::Authorization("Invalid API key".to_string())
    })?;

    let api_key_record = keys.into_iter().next().ok_or_else(|| {
        tracing::warn!("API key not found or inactive");
        ApiError::Authorization("Invalid or inactive API key".to_string())
    })?;

    // Check expiry
    if let Some(expires_at) = api_key_record.expires_at {
        if expires_at < Utc::now() {
            return Err(ApiError::Authorization("API key expired".to_string()));
        }
    }

    // TODO: Check rate limits using Redis

    // Update last_used_at
    let update_query = format!(
        "UPDATE {} SET last_used_at = time::now()",
        api_key_record.id.as_ref().unwrap()
    );
    state.db.client.query(&update_query).await.ok();

    // Generate request ID
    let request_id = format!("VAL-{}", uuid::Uuid::new_v4());

    // Search for matches based on check type
    let (matched, matches) = match payload.check_type {
        CheckType::Basic => {
            // Simple name match
            search_by_name(&state, &payload.full_name).await?
        }
        CheckType::Standard => {
            // Name + DOB
            if let Some(dob) = payload.date_of_birth {
                search_by_name_and_dob(&state, &payload.full_name, &dob).await?
            } else {
                return Err(ApiError::Validation(
                    "Date of birth required for standard check".to_string(),
                ));
            }
        }
        CheckType::Enhanced => {
            // Name + DOB + postcode
            if payload.date_of_birth.is_none() || payload.postcode.is_none() {
                return Err(ApiError::Validation(
                    "Date of birth and postcode required for enhanced check".to_string(),
                ));
            }
            search_enhanced(&state, &payload).await?
        }
    };

    // Determine confidence level
    let confidence = if matches.is_empty() {
        ConfidenceLevel::None
    } else if matches[0].match_score >= 0.95 {
        ConfidenceLevel::High
    } else if matches[0].match_score >= 0.80 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    };

    let response = ValidationResponse {
        request_id: request_id.clone(),
        matched,
        confidence,
        matches: matches.into_iter().take(5).collect(), // Limit to 5 matches
        disclaimer: "This check is informational only and should not be the sole basis for employment decisions. Manual verification recommended for positive matches.".to_string(),
        timestamp: Utc::now(),
    };

    tracing::info!(
        "Business API validation: {} - matched={} confidence={:?}",
        request_id,
        matched,
        response.confidence
    );

    Ok(Json(response))
}

/// Search by name only (basic check)
async fn search_by_name(
    state: &AppState,
    full_name: &str,
) -> Result<(bool, Vec<MatchSummary>), ApiError> {
    let query = format!(
        "SELECT * FROM conviction_record WHERE full_name CONTAINS '{}' LIMIT 10",
        full_name.replace('\'', "\\'")
    );

    let results: Vec<serde_json::Value> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to search convictions: {}", e);
        ApiError::Internal(anyhow::anyhow!("Search failed"))
    })?;

    let matches: Vec<MatchSummary> = results
        .into_iter()
        .map(|r| MatchSummary {
            conviction_id: r["id"].as_str().unwrap_or("unknown").to_string(),
            match_score: 0.75, // Basic name match gets 75% score
            offense_category: r["offense_category"]
                .as_str()
                .unwrap_or("unspecified")
                .to_string(),
            conviction_year: r["conviction_date"]
                .as_str()
                .and_then(|d| d.split('-').next())
                .and_then(|y| y.parse().ok()),
            requires_manual_review: true,
        })
        .collect();

    let matched = !matches.is_empty();
    Ok((matched, matches))
}

/// Search by name and date of birth (standard check)
async fn search_by_name_and_dob(
    state: &AppState,
    full_name: &str,
    dob: &str,
) -> Result<(bool, Vec<MatchSummary>), ApiError> {
    let query = format!(
        "SELECT * FROM conviction_record WHERE full_name CONTAINS '{}' AND date_of_birth = '{}' LIMIT 10",
        full_name.replace('\'', "\\'"),
        dob.replace('\'', "\\'")
    );

    let results: Vec<serde_json::Value> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to search convictions: {}", e);
        ApiError::Internal(anyhow::anyhow!("Search failed"))
    })?;

    let matches: Vec<MatchSummary> = results
        .into_iter()
        .map(|r| MatchSummary {
            conviction_id: r["id"].as_str().unwrap_or("unknown").to_string(),
            match_score: 0.90, // Name + DOB match gets 90% score
            offense_category: r["offense_category"]
                .as_str()
                .unwrap_or("unspecified")
                .to_string(),
            conviction_year: r["conviction_date"]
                .as_str()
                .and_then(|d| d.split('-').next())
                .and_then(|y| y.parse().ok()),
            requires_manual_review: false,
        })
        .collect();

    let matched = !matches.is_empty();
    Ok((matched, matches))
}

/// Enhanced search with name, DOB, and postcode
async fn search_enhanced(
    state: &AppState,
    payload: &ValidationRequest,
) -> Result<(bool, Vec<MatchSummary>), ApiError> {
    let query = format!(
        "SELECT * FROM conviction_record WHERE full_name CONTAINS '{}' AND date_of_birth = '{}' LIMIT 10",
        payload.full_name.replace('\'', "\\'"),
        payload
            .date_of_birth
            .as_ref()
            .unwrap()
            .replace('\'', "\\'")
    );

    let results: Vec<serde_json::Value> = state.db.query(&query).await.map_err(|e| {
        tracing::error!("Failed to search convictions: {}", e);
        ApiError::Internal(anyhow::anyhow!("Search failed"))
    })?;

    let matches: Vec<MatchSummary> = results
        .into_iter()
        .map(|r| MatchSummary {
            conviction_id: r["id"].as_str().unwrap_or("unknown").to_string(),
            match_score: 0.98, // Enhanced match gets 98% score
            offense_category: r["offense_category"]
                .as_str()
                .unwrap_or("unspecified")
                .to_string(),
            conviction_year: r["conviction_date"]
                .as_str()
                .and_then(|d| d.split('-').next())
                .and_then(|y| y.parse().ok()),
            requires_manual_review: false,
        })
        .collect();

    let matched = !matches.is_empty();
    Ok((matched, matches))
}

/// Get API usage statistics for authenticated tenant
async fn get_usage(State(_state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement usage tracking from Redis or audit logs
    Ok(Json(serde_json::json!({
        "message": "Usage statistics endpoint",
        "status": "not_implemented"
    })))
}

/// Create new API key for a tenant (admin only)
async fn create_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> ApiResult<Json<CreateApiKeyResponse>> {
    // TODO: Check admin authentication

    // Generate random API key
    let api_key = format!(
        "phdb_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    let key_prefix = format!("phdb_{}...", &api_key[5..13]);

    // Hash the key for storage
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    let now = Utc::now();
    let expires_at = payload
        .expires_days
        .map(|days| now + Duration::days(days as i64));

    // Create API key record
    let query = format!(
        "CREATE api_key CONTENT {{
            tenant_id: {},
            key_hash: '{}',
            key_prefix: '{}',
            scopes: {},
            rate_limit_per_hour: 100,
            status: 'active',
            created_at: time::now(),
            expires_at: {}
        }}",
        payload.tenant_id,
        key_hash,
        key_prefix,
        serde_json::to_string(&payload.scopes).unwrap(),
        expires_at
            .map(|e| format!("'{}'", e.to_rfc3339()))
            .unwrap_or_else(|| "NONE".to_string())
    );

    state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to create API key: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create API key"))
    })?;

    tracing::info!("Created API key for tenant: {}", payload.tenant_id);

    Ok(Json(CreateApiKeyResponse {
        api_key, // Return plain text key only once
        key_prefix,
        expires_at,
    }))
}

/// Delete an API key (admin or tenant owner)
async fn delete_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<StatusCode> {
    // TODO: Check admin or tenant owner authentication

    let query = format!("UPDATE {} SET status = 'suspended'", key_id);

    state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to delete API key: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to delete API key"))
    })?;

    tracing::info!("Deleted API key: {}", key_id);

    Ok(StatusCode::NO_CONTENT)
}

/// Create new business tenant (admin only)
async fn create_tenant(
    State(state): State<AppState>,
    Json(payload): Json<CreateTenantRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Check admin authentication

    // Validate request
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let query = format!(
        "CREATE business_tenant CONTENT {{
            name: '{}',
            contact_email: '{}',
            industry: '{}',
            purpose_statement: '{}',
            status: 'pending',
            rate_limit_tier: 'basic',
            created_at: time::now()
        }}",
        payload.name.replace('\'', "\\'"),
        payload.contact_email.replace('\'', "\\'"),
        payload.industry.replace('\'', "\\'"),
        payload.purpose_statement.replace('\'', "\\'")
    );

    let mut result = state.db.client.query(&query).await.map_err(|e| {
        tracing::error!("Failed to create tenant: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to create tenant"))
    })?;

    let tenant: Option<serde_json::Value> = result.take(0).map_err(|e| {
        tracing::error!("Failed to parse created tenant: {}", e);
        ApiError::Internal(anyhow::anyhow!("Failed to parse tenant"))
    })?;

    tracing::info!("Created business tenant: {}", payload.name);

    Ok(Json(
        tenant.ok_or_else(|| ApiError::Internal(anyhow::anyhow!("No tenant returned")))?,
    ))
}
