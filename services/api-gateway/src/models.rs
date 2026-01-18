use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================================================
// USER & AUTH MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user ID
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,         // expiration time
    pub iat: usize,         // issued at
}

// ============================================================================
// REPORT MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub category: String,
    pub incident_date: Option<DateTime<Utc>>,
    pub street_name: Option<String>,
    pub city: Option<String>,
    pub postcode_district: Option<String>,
    pub status: ReportStatus,
    pub visibility_tier: VisibilityTier,
    pub harm_risk: HarmRisk,
    pub source_refs: Vec<String>,
    pub submitted_by: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Draft,
    Submitted,
    Triage,
    UnderReview,
    Approved,
    Rejected,
    NeedsMoreInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityTier {
    Public,
    LoggedIn,
    VerifiedBusiness,
    Reviewers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HarmRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReportRequest {
    #[validate(length(min = 5, max = 200))]
    pub title: String,
    #[validate(length(min = 10))]
    pub description: String,
    pub category: String,
    pub incident_date: Option<DateTime<Utc>>,
    pub street_name: Option<String>,
    pub city: Option<String>,
    pub postcode_district: Option<String>,
}

// ============================================================================
// MISSING PERSON ALERT MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingPersonAlert {
    pub id: Option<String>,
    pub alert_id: String,
    pub full_name: String,
    pub age: u32,
    pub description: String,
    pub last_seen_date: DateTime<Utc>,
    pub photo_url: Option<String>,
    pub contact_info: String,
    pub status: AlertStatus,
    pub priority: AlertPriority,
    pub active_until: DateTime<Utc>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Draft,
    Verified,
    Active,
    Expired,
    Resolved,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertPriority {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// FACE SEARCH MODELS
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct FaceSearchRequest {
    pub image: String,  // base64 encoded
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FaceSearchResponse {
    pub candidates: Vec<FaceCandidate>,
    pub disclaimer: String,
    pub confidence_bucket: String,
}

#[derive(Debug, Serialize)]
pub struct FaceCandidate {
    pub id: String,
    pub confidence: f32,
    pub requires_verification: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceSearchAudit {
    pub id: Option<String>,
    pub actor_id: Option<String>,
    pub actor_ip_hash: String,
    pub timestamp: DateTime<Utc>,
    pub result_count: usize,
    pub confidence_bucket: String,
    pub purpose: Option<String>,
}

// ============================================================================
// BUSINESS API MODELS
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct ValidateRequest {
    pub identifier: Identifier,
    pub purpose: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Identifier {
    #[serde(rename = "document")]
    Document {
        document_number: String,
        dob: String,
    },
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub request_id: String,
    pub status: ValidationStatus,
    pub risk_level: RiskLevel,
    pub reason_codes: Vec<String>,
    pub decision_expires_at: DateTime<Utc>,
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Verified,
    Unverified,
    ReviewRequired,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

// ============================================================================
// PAGINATION
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u64,
}
