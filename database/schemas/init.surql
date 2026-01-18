-- Predator Hunters Platform - SurrealDB Schema
-- Initial schema setup for all core tables

-- ============================================================================
-- USERS & AUTHENTICATION
-- ============================================================================

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string ASSERT string::len($value) > 2;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD roles ON user TYPE array<string> DEFAULT [];
DEFINE FIELD status ON user TYPE string DEFAULT 'active' ASSERT $value IN ['active', 'suspended', 'deleted'];
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD last_login ON user TYPE option<datetime>;
DEFINE FIELD mfa_enabled ON user TYPE bool DEFAULT false;
DEFINE FIELD mfa_secret ON user TYPE option<string>;
DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;
DEFINE INDEX idx_user_username ON user FIELDS username UNIQUE;

-- Sessions
DEFINE TABLE session SCHEMAFULL;
DEFINE FIELD user_id ON session TYPE record<user>;
DEFINE FIELD token_hash ON session TYPE string;
DEFINE FIELD expires_at ON session TYPE datetime;
DEFINE FIELD ip_address ON session TYPE string;
DEFINE FIELD user_agent ON session TYPE option<string>;
DEFINE FIELD created_at ON session TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_session_token ON session FIELDS token_hash UNIQUE;
DEFINE INDEX idx_session_expiry ON session FIELDS expires_at;

-- Role Bindings
DEFINE TABLE role_binding SCHEMAFULL;
DEFINE FIELD user_id ON role_binding TYPE record<user>;
DEFINE FIELD role ON role_binding TYPE string;
DEFINE FIELD resource_type ON role_binding TYPE option<string>;
DEFINE FIELD resource_id ON role_binding TYPE option<string>;
DEFINE FIELD granted_by ON role_binding TYPE record<user>;
DEFINE FIELD granted_at ON role_binding TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON role_binding TYPE option<datetime>;
DEFINE INDEX idx_role_user ON role_binding FIELDS user_id, role;

-- ============================================================================
-- REPORTS & EVIDENCE
-- ============================================================================

DEFINE TABLE report SCHEMAFULL;
DEFINE FIELD title ON report TYPE string;
DEFINE FIELD description ON report TYPE string;
DEFINE FIELD category ON report TYPE string;
DEFINE FIELD incident_date ON report TYPE option<datetime>;
DEFINE FIELD incident_location ON report TYPE option<geometry<point>>;
DEFINE FIELD street_name ON report TYPE option<string>;
DEFINE FIELD city ON report TYPE option<string>;
DEFINE FIELD postcode_district ON report TYPE option<string>;
DEFINE FIELD status ON report TYPE string DEFAULT 'draft' ASSERT $value IN ['draft', 'submitted', 'triage', 'under_review', 'approved', 'rejected', 'needs_more_info'];
DEFINE FIELD visibility_tier ON report TYPE string DEFAULT 'reviewers' ASSERT $value IN ['public', 'logged_in', 'verified_business', 'reviewers'];
DEFINE FIELD harm_risk ON report TYPE string DEFAULT 'low' ASSERT $value IN ['low', 'medium', 'high'];
DEFINE FIELD source_refs ON report TYPE array<string> DEFAULT [];
DEFINE FIELD submitted_by ON report TYPE option<record<user>>;
DEFINE FIELD submitted_at ON report TYPE option<datetime>;
DEFINE FIELD created_at ON report TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON report TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_report_status ON report FIELDS status, created_at;
DEFINE INDEX idx_report_submitter ON report FIELDS submitted_by;

-- Evidence
DEFINE TABLE evidence SCHEMAFULL;
DEFINE FIELD report_id ON evidence TYPE record<report>;
DEFINE FIELD media_asset_id ON evidence TYPE option<record<media_asset>>;
DEFINE FIELD evidence_type ON evidence TYPE string ASSERT $value IN ['video', 'image', 'document', 'audio', 'text'];
DEFINE FIELD description ON evidence TYPE string;
DEFINE FIELD collected_at ON evidence TYPE option<datetime>;
DEFINE FIELD chain_of_custody ON evidence TYPE array<object> DEFAULT [];
DEFINE FIELD sealed ON evidence TYPE bool DEFAULT false;
DEFINE FIELD sealed_at ON evidence TYPE option<datetime>;
DEFINE FIELD sealed_by ON evidence TYPE option<record<user>>;
DEFINE FIELD created_at ON evidence TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_evidence_report ON evidence FIELDS report_id;

-- Media Assets
DEFINE TABLE media_asset SCHEMAFULL;
DEFINE FIELD filename ON media_asset TYPE string;
DEFINE FIELD mime_type ON media_asset TYPE string;
DEFINE FIELD size_bytes ON media_asset TYPE int;
DEFINE FIELD storage_path ON media_asset TYPE string;
DEFINE FIELD storage_bucket ON media_asset TYPE string;
DEFINE FIELD checksum_sha256 ON media_asset TYPE string;
DEFINE FIELD variants ON media_asset TYPE object DEFAULT {};
DEFINE FIELD thumbnail_path ON media_asset TYPE option<string>;
DEFINE FIELD preview_clip_path ON media_asset TYPE option<string>;
DEFINE FIELD thumbnail_strip_path ON media_asset TYPE option<string>;
DEFINE FIELD duration_seconds ON media_asset TYPE option<float>;
DEFINE FIELD width ON media_asset TYPE option<int>;
DEFINE FIELD height ON media_asset TYPE option<int>;
DEFINE FIELD processing_status ON media_asset TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'processing', 'completed', 'failed'];
DEFINE FIELD virus_scan_status ON media_asset TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'clean', 'infected', 'error'];
DEFINE FIELD uploaded_by ON media_asset TYPE record<user>;
DEFINE FIELD uploaded_at ON media_asset TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_media_uploader ON media_asset FIELDS uploaded_by;
DEFINE INDEX idx_media_processing ON media_asset FIELDS processing_status;

-- ============================================================================
-- REVIEW & MODERATION
-- ============================================================================

DEFINE TABLE review_task SCHEMAFULL;
DEFINE FIELD report_id ON review_task TYPE record<report>;
DEFINE FIELD task_type ON review_task TYPE string ASSERT $value IN ['triage', 'evidence_review', 'source_validation', 'harm_assessment', 'publication_approval'];
DEFINE FIELD status ON review_task TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'assigned', 'in_progress', 'completed', 'escalated'];
DEFINE FIELD priority ON review_task TYPE int DEFAULT 5 ASSERT $value >= 1 AND $value <= 10;
DEFINE FIELD assigned_to ON review_task TYPE option<record<user>>;
DEFINE FIELD assigned_at ON review_task TYPE option<datetime>;
DEFINE FIELD completed_at ON review_task TYPE option<datetime>;
DEFINE FIELD decision ON review_task TYPE option<string>;
DEFINE FIELD decision_notes ON review_task TYPE option<string>;
DEFINE FIELD created_at ON review_task TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_review_task_status ON review_task FIELDS status, priority;
DEFINE INDEX idx_review_task_assignee ON review_task FIELDS assigned_to;

-- Publishable Items
DEFINE TABLE publishable_item SCHEMAFULL;
DEFINE FIELD slug ON publishable_item TYPE string;
DEFINE FIELD title ON publishable_item TYPE string;
DEFINE FIELD content_type ON publishable_item TYPE string ASSERT $value IN ['conviction', 'news_article', 'investigation', 'survivor_story'];
DEFINE FIELD content ON publishable_item TYPE string;
DEFINE FIELD summary ON publishable_item TYPE option<string>;
DEFINE FIELD source_report_id ON publishable_item TYPE option<record<report>>;
DEFINE FIELD status ON publishable_item TYPE string DEFAULT 'draft' ASSERT $value IN ['draft', 'published', 'corrected', 'taken_down', 'archived'];
DEFINE FIELD visibility_tier ON publishable_item TYPE string DEFAULT 'public';
DEFINE FIELD published_at ON publishable_item TYPE option<datetime>;
DEFINE FIELD published_by ON publishable_item TYPE option<record<user>>;
DEFINE FIELD corrections ON publishable_item TYPE array<object> DEFAULT [];
DEFINE FIELD takedown_reason ON publishable_item TYPE option<string>;
DEFINE FIELD created_at ON publishable_item TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON publishable_item TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_item_slug ON publishable_item FIELDS slug UNIQUE;
DEFINE INDEX idx_item_status ON publishable_item FIELDS status, published_at;

-- ============================================================================
-- MAP ENTRIES
-- ============================================================================

DEFINE TABLE map_entry SCHEMAFULL;
DEFINE FIELD geometry ON map_entry TYPE geometry<point>;
DEFINE FIELD precision_class ON map_entry TYPE string ASSERT $value IN ['street_segment', 'street_center', 'area'];
DEFINE FIELD display_policy ON map_entry TYPE string DEFAULT 'standard';
DEFINE FIELD linked_item_id ON map_entry TYPE option<record<publishable_item>>;
DEFINE FIELD linked_conviction_id ON map_entry TYPE option<record<conviction_record>>;
DEFINE FIELD street_name ON map_entry TYPE string;
DEFINE FIELD city ON map_entry TYPE string;
DEFINE FIELD postcode_district ON map_entry TYPE option<string>;
DEFINE FIELD visibility_tier ON map_entry TYPE string DEFAULT 'public';
DEFINE FIELD harm_risk ON map_entry TYPE string DEFAULT 'low';
DEFINE FIELD verified ON map_entry TYPE bool DEFAULT false;
DEFINE FIELD created_at ON map_entry TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_map_geometry ON map_entry FIELDS geometry;
DEFINE INDEX idx_map_visibility ON map_entry FIELDS visibility_tier, harm_risk;

-- ============================================================================
-- CONVICTION RECORDS (Legacy compatibility + enhanced)
-- ============================================================================

DEFINE TABLE conviction_record SCHEMAFULL;
DEFINE FIELD full_name ON conviction_record TYPE string;
DEFINE FIELD offense_type ON conviction_record TYPE string;
DEFINE FIELD conviction_date ON conviction_record TYPE string;
DEFINE FIELD court_name ON conviction_record TYPE string;
DEFINE FIELD sentence ON conviction_record TYPE string;
DEFINE FIELD street_name ON conviction_record TYPE string;
DEFINE FIELD city ON conviction_record TYPE string;
DEFINE FIELD postcode_district ON conviction_record TYPE option<string>;
DEFINE FIELD latitude ON conviction_record TYPE option<float>;
DEFINE FIELD longitude ON conviction_record TYPE option<float>;
DEFINE FIELD source ON conviction_record TYPE string;
DEFINE FIELD interview_consent ON conviction_record TYPE bool DEFAULT false;
DEFINE FIELD interview_date ON conviction_record TYPE option<string>;
DEFINE FIELD notes ON conviction_record TYPE string DEFAULT '';
DEFINE FIELD verification_status ON conviction_record TYPE string DEFAULT 'unverified';
DEFINE FIELD verification_notes ON conviction_record TYPE option<string>;
DEFINE FIELD verified_by ON conviction_record TYPE option<record<user>>;
DEFINE FIELD verified_at ON conviction_record TYPE option<datetime>;
DEFINE FIELD created_at ON conviction_record TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON conviction_record TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_conviction_verification ON conviction_record FIELDS verification_status;

-- ============================================================================
-- MISSING PERSON ALERTS
-- ============================================================================

DEFINE TABLE missing_person_alert SCHEMAFULL;
DEFINE FIELD alert_id ON missing_person_alert TYPE string;
DEFINE FIELD full_name ON missing_person_alert TYPE string;
DEFINE FIELD age ON missing_person_alert TYPE int;
DEFINE FIELD description ON missing_person_alert TYPE string;
DEFINE FIELD last_seen_location ON missing_person_alert TYPE option<geometry<point>>;
DEFINE FIELD last_seen_date ON missing_person_alert TYPE datetime;
DEFINE FIELD photo_url ON missing_person_alert TYPE option<string>;
DEFINE FIELD contact_info ON missing_person_alert TYPE string;
DEFINE FIELD status ON missing_person_alert TYPE string DEFAULT 'draft' ASSERT $value IN ['draft', 'verified', 'active', 'expired', 'resolved', 'archived'];
DEFINE FIELD priority ON missing_person_alert TYPE string DEFAULT 'medium' ASSERT $value IN ['low', 'medium', 'high', 'critical'];
DEFINE FIELD geofence_radius_km ON missing_person_alert TYPE option<float>;
DEFINE FIELD active_until ON missing_person_alert TYPE datetime;
DEFINE FIELD created_by ON missing_person_alert TYPE record<user>;
DEFINE FIELD created_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD resolved_at ON missing_person_alert TYPE option<datetime>;
DEFINE FIELD archived_at ON missing_person_alert TYPE option<datetime>;
DEFINE INDEX idx_alert_status ON missing_person_alert FIELDS status, active_until;
DEFINE INDEX idx_alert_active ON missing_person_alert FIELDS status, priority;

-- ============================================================================
-- SURVIVOR STORIES
-- ============================================================================

DEFINE TABLE survivor_story SCHEMAFULL;
DEFINE FIELD title ON survivor_story TYPE string;
DEFINE FIELD content ON survivor_story TYPE string;
DEFINE FIELD author_pseudonym ON survivor_story TYPE option<string>;
DEFINE FIELD submitted_by ON survivor_story TYPE option<record<user>>;
DEFINE FIELD consent_given ON survivor_story TYPE bool DEFAULT false;
DEFINE FIELD consent_date ON survivor_story TYPE option<datetime>;
DEFINE FIELD status ON survivor_story TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'reviewing', 'approved', 'rejected', 'published'];
DEFINE FIELD published_at ON survivor_story TYPE option<datetime>;
DEFINE FIELD visibility ON survivor_story TYPE string DEFAULT 'private';
DEFINE FIELD created_at ON survivor_story TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON survivor_story TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_story_status ON survivor_story FIELDS status;

-- ============================================================================
-- FACE RECOGNITION (Curated Dataset Only)
-- ============================================================================

DEFINE TABLE face_dataset_item SCHEMAFULL;
DEFINE FIELD linked_conviction_id ON face_dataset_item TYPE option<record<conviction_record>>;
DEFINE FIELD image_url ON face_dataset_item TYPE string;
DEFINE FIELD embedding_ref ON face_dataset_item TYPE string;
DEFINE FIELD source_type ON face_dataset_item TYPE string ASSERT $value IN ['court_record', 'public_record', 'consented'];
DEFINE FIELD consent_obtained ON face_dataset_item TYPE bool;
DEFINE FIELD added_by ON face_dataset_item TYPE record<user>;
DEFINE FIELD added_at ON face_dataset_item TYPE datetime DEFAULT time::now();
DEFINE FIELD verified ON face_dataset_item TYPE bool DEFAULT false;
DEFINE INDEX idx_face_dataset_conviction ON face_dataset_item FIELDS linked_conviction_id;

-- Face Search Audit (NO IMAGES OR EMBEDDINGS STORED)
DEFINE TABLE face_search_audit SCHEMAFULL;
DEFINE FIELD actor_id ON face_search_audit TYPE option<record<user>>;
DEFINE FIELD actor_ip_hash ON face_search_audit TYPE string;
DEFINE FIELD timestamp ON face_search_audit TYPE datetime DEFAULT time::now();
DEFINE FIELD result_count ON face_search_audit TYPE int;
DEFINE FIELD confidence_bucket ON face_search_audit TYPE string;
DEFINE FIELD purpose ON face_search_audit TYPE option<string>;
DEFINE FIELD tenant_id ON face_search_audit TYPE option<record<business_tenant>>;
DEFINE INDEX idx_face_search_time ON face_search_audit FIELDS timestamp;
DEFINE INDEX idx_face_search_actor ON face_search_audit FIELDS actor_id, timestamp;

-- ============================================================================
-- BUSINESS API
-- ============================================================================

DEFINE TABLE business_tenant SCHEMAFULL;
DEFINE FIELD name ON business_tenant TYPE string;
DEFINE FIELD contact_email ON business_tenant TYPE string;
DEFINE FIELD industry ON business_tenant TYPE string;
DEFINE FIELD purpose_statement ON business_tenant TYPE string;
DEFINE FIELD status ON business_tenant TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'approved', 'suspended', 'terminated'];
DEFINE FIELD approved_by ON business_tenant TYPE option<record<user>>;
DEFINE FIELD approved_at ON business_tenant TYPE option<datetime>;
DEFINE FIELD rate_limit_tier ON business_tenant TYPE string DEFAULT 'basic' ASSERT $value IN ['basic', 'standard', 'premium'];
DEFINE FIELD created_at ON business_tenant TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_tenant_status ON business_tenant FIELDS status;

-- API Keys
DEFINE TABLE api_key SCHEMAFULL;
DEFINE FIELD tenant_id ON api_key TYPE record<business_tenant>;
DEFINE FIELD key_hash ON api_key TYPE string;
DEFINE FIELD key_prefix ON api_key TYPE string;
DEFINE FIELD scopes ON api_key TYPE array<string> DEFAULT ['validate'];
DEFINE FIELD rate_limit_per_hour ON api_key TYPE int DEFAULT 100;
DEFINE FIELD allowlist_ips ON api_key TYPE array<string> DEFAULT [];
DEFINE FIELD status ON api_key TYPE string DEFAULT 'active' ASSERT $value IN ['active', 'revoked', 'expired'];
DEFINE FIELD last_used_at ON api_key TYPE option<datetime>;
DEFINE FIELD created_by ON api_key TYPE record<user>;
DEFINE FIELD created_at ON api_key TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON api_key TYPE option<datetime>;
DEFINE INDEX idx_api_key_hash ON api_key FIELDS key_hash UNIQUE;
DEFINE INDEX idx_api_key_tenant ON api_key FIELDS tenant_id;

-- API Audit Log
DEFINE TABLE api_audit_log SCHEMAFULL;
DEFINE FIELD tenant_id ON api_audit_log TYPE option<record<business_tenant>>;
DEFINE FIELD api_key_id ON api_audit_log TYPE option<record<api_key>>;
DEFINE FIELD endpoint ON api_audit_log TYPE string;
DEFINE FIELD method ON api_audit_log TYPE string;
DEFINE FIELD status_code ON api_audit_log TYPE int;
DEFINE FIELD request_id ON api_audit_log TYPE string;
DEFINE FIELD ip_address ON api_audit_log TYPE string;
DEFINE FIELD user_agent ON api_audit_log TYPE option<string>;
DEFINE FIELD response_time_ms ON api_audit_log TYPE int;
DEFINE FIELD timestamp ON api_audit_log TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_audit_timestamp ON api_audit_log FIELDS timestamp;
DEFINE INDEX idx_audit_tenant ON api_audit_log FIELDS tenant_id, timestamp;

-- ============================================================================
-- TAKEDOWNS & CORRECTIONS
-- ============================================================================

DEFINE TABLE takedown_request SCHEMAFULL;
DEFINE FIELD item_id ON takedown_request TYPE record<publishable_item>;
DEFINE FIELD requested_by ON takedown_request TYPE option<record<user>>;
DEFINE FIELD requester_email ON takedown_request TYPE option<string>;
DEFINE FIELD reason ON takedown_request TYPE string;
DEFINE FIELD evidence ON takedown_request TYPE option<string>;
DEFINE FIELD status ON takedown_request TYPE string DEFAULT 'pending' ASSERT $value IN ['pending', 'reviewing', 'approved', 'denied'];
DEFINE FIELD reviewed_by ON takedown_request TYPE option<record<user>>;
DEFINE FIELD reviewed_at ON takedown_request TYPE option<datetime>;
DEFINE FIELD decision ON takedown_request TYPE option<string>;
DEFINE FIELD decision_notes ON takedown_request TYPE option<string>;
DEFINE FIELD created_at ON takedown_request TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_takedown_status ON takedown_request FIELDS status;

-- Correction Log
DEFINE TABLE correction_log SCHEMAFULL;
DEFINE FIELD item_id ON correction_log TYPE record<publishable_item>;
DEFINE FIELD correction_type ON correction_log TYPE string;
DEFINE FIELD old_value ON correction_log TYPE option<string>;
DEFINE FIELD new_value ON correction_log TYPE option<string>;
DEFINE FIELD reason ON correction_log TYPE string;
DEFINE FIELD corrected_by ON correction_log TYPE record<user>;
DEFINE FIELD corrected_at ON correction_log TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_correction_item ON correction_log FIELDS item_id, corrected_at;

-- ============================================================================
-- SYSTEM AUDIT LOG
-- ============================================================================

DEFINE TABLE system_audit_log SCHEMAFULL;
DEFINE FIELD event_type ON system_audit_log TYPE string;
DEFINE FIELD actor_id ON system_audit_log TYPE option<record<user>>;
DEFINE FIELD actor_type ON system_audit_log TYPE string DEFAULT 'user';
DEFINE FIELD resource_type ON system_audit_log TYPE option<string>;
DEFINE FIELD resource_id ON system_audit_log TYPE option<string>;
DEFINE FIELD action ON system_audit_log TYPE string;
DEFINE FIELD outcome ON system_audit_log TYPE string;
DEFINE FIELD ip_address ON system_audit_log TYPE option<string>;
DEFINE FIELD user_agent ON system_audit_log TYPE option<string>;
DEFINE FIELD metadata ON system_audit_log TYPE object DEFAULT {};
DEFINE FIELD timestamp ON system_audit_log TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_audit_event_type ON system_audit_log FIELDS event_type, timestamp;
DEFINE INDEX idx_audit_actor ON system_audit_log FIELDS actor_id, timestamp;
