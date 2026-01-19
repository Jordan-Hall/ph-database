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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAlertRequest {
    #[validate(length(min = 3, max = 100))]
    pub full_name: String,
    #[validate(range(min = 0, max = 150))]
    pub age: u32,
    #[validate(length(min = 20, max = 2000))]
    pub description: String,
    pub last_seen_date: DateTime<Utc>,
    pub last_seen_location: Option<serde_json::Value>,  // GeoJSON point
    pub photo_url: Option<String>,
    #[validate(length(min = 10, max = 200))]
    pub contact_info: String,
    pub priority: AlertPriority,
    pub geofence_radius_km: Option<f64>,
    pub active_days: Option<u32>,  // Default 30 days
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertStatusRequest {
    pub status: AlertStatus,
    pub resolution_notes: Option<String>,
}

fn default_alert_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    #[serde(default = "default_alert_limit")]
    pub limit: u32,
    pub status: Option<String>,
    pub priority: Option<String>,
}

// ============================================================================
// MAP ENTRY MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub id: Option<String>,
    pub geometry: serde_json::Value,  // GeoJSON Point
    pub precision_class: PrecisionClass,
    pub display_policy: DisplayPolicy,
    pub linked_item_id: Option<String>,
    pub linked_conviction_id: Option<String>,
    pub street_name: String,
    pub city: String,
    pub postcode_district: Option<String>,
    pub visibility_tier: VisibilityTier,
    pub harm_risk: HarmRisk,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrecisionClass {
    Exact,      // Full address
    Street,     // Street name only
    District,   // Postcode district
    City,       // City level
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayPolicy {
    Standard,   // Show as-is
    Fuzzy,      // Show approximate location
    Hidden,     // Don't display on public map
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMapEntryRequest {
    pub longitude: f64,
    pub latitude: f64,
    pub precision_class: PrecisionClass,
    pub display_policy: DisplayPolicy,
    pub linked_item_id: Option<String>,
    pub linked_conviction_id: Option<String>,
    #[validate(length(min = 3, max = 200))]
    pub street_name: String,
    #[validate(length(min = 2, max = 100))]
    pub city: String,
    pub postcode_district: Option<String>,
    pub visibility_tier: VisibilityTier,
    pub harm_risk: HarmRisk,
}

#[derive(Debug, Deserialize)]
pub struct MapBoundsQuery {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
    pub visibility_tier: Option<String>,
    pub harm_risk: Option<String>,
    #[serde(default = "default_map_limit")]
    pub limit: u32,
}

fn default_map_limit() -> u32 {
    200
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
// EVIDENCE & MEDIA MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Option<String>,
    pub report_id: String,
    pub media_asset_id: Option<String>,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub collected_at: Option<DateTime<Utc>>,
    pub chain_of_custody: Vec<serde_json::Value>,
    pub sealed: bool,
    pub sealed_at: Option<DateTime<Utc>>,
    pub sealed_by: Option<String>,
    pub submitted_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Video,
    Image,
    Document,
    Testimony,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub storage_path: String,
    pub storage_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub duration_seconds: Option<f32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub transcoding_status: TranscodingStatus,
    pub checksum_sha256: String,
    pub uploaded_by: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranscodingStatus {
    Pending,
    Processing,
    Complete,
    Failed,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UploadEvidenceRequest {
    #[validate(length(min = 1))]
    pub evidence_type: String,
    #[validate(length(min = 5, max = 1000))]
    pub description: String,
    pub collected_at: Option<DateTime<Utc>>,
    pub file_base64: Option<String>,  // For small files (images, documents)
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EvidenceResponse {
    pub evidence: Evidence,
    pub upload_url: Option<String>,  // For large files (videos) - presigned S3 URL
}

// ============================================================================
// AUDIT LOG MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Option<String>,
    pub actor_id: String,
    pub actor_username: String,
    pub actor_ip_hash: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditLogRequest {
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// PUBLISHABLE ITEM MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishableItem {
    pub id: Option<String>,
    pub slug: String,
    pub title: String,
    pub content_type: String,
    pub content: String,
    pub summary: Option<String>,
    pub source_report_id: Option<String>,
    pub status: PublishStatus,
    pub visibility_tier: VisibilityTier,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<String>,
    pub corrections: Vec<serde_json::Value>,
    pub takedown_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    Draft,
    Published,
    Withdrawn,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PublishRequest {
    #[validate(length(min = 3, max = 100))]
    pub slug: String,
    #[validate(length(min = 10, max = 200))]
    pub title: String,
    pub content_type: String,
    #[validate(length(min = 100))]
    pub content: String,
    #[validate(length(max = 500))]
    pub summary: Option<String>,
    pub visibility_tier: VisibilityTier,
}

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub item: PublishableItem,
    pub public_url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CorrectionRequest {
    pub correction_type: String,
    #[validate(length(min = 10, max = 1000))]
    pub reason: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionLog {
    pub id: Option<String>,
    pub item_id: String,
    pub correction_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: String,
    pub corrected_by: String,
    pub corrected_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TakedownRequest {
    #[validate(length(min = 20, max = 2000))]
    pub reason: String,
    pub requester_email: Option<String>,
    pub evidence_description: Option<String>,
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
