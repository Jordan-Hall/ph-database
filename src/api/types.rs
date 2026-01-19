use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Authentication Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

// ============================================================================
// Report Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub subject_name: String,
    pub offense_type: String,
    pub harm_risk: String,
    pub status: String,
    pub visibility: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub title: String,
    pub description: String,
    pub subject_name: String,
    pub offense_type: String,
    pub harm_risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReportsRequest {
    pub query: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

// ============================================================================
// Alert Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Option<String>,
    pub alert_id: String,
    pub full_name: String,
    pub age: Option<i32>,
    pub last_seen_location: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertRequest {
    pub full_name: String,
    pub age: Option<i32>,
    pub last_seen_location: String,
    pub description: String,
    pub priority: String,
    pub contact_info: Option<String>,
}

// ============================================================================
// Story Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub author_pseudonym: Option<String>,
    pub status: String,
    pub trigger_warning: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitStoryRequest {
    pub title: String,
    pub content: String,
    pub author_pseudonym: Option<String>,
    pub consent_given: bool,
    pub trigger_warning: Option<String>,
}

// ============================================================================
// Map Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub id: Option<String>,
    pub report_id: String,
    pub location_lat: f64,
    pub location_lon: f64,
    pub precision_class: String,
    pub display_policy: String,
    pub harm_risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBoundsQuery {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

// ============================================================================
// Published Item Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedItem {
    pub id: Option<String>,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Business API Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessTenant {
    pub id: Option<String>,
    pub company_name: String,
    pub contact_email: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Option<String>,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Paginated Response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}
