-- Predator Hunters Platform - Enhanced SurrealDB Schema
-- Leveraging SurrealDB native features: ML, RLAC, Graph, Full-text Search

-- ============================================================================
-- AUTHENTICATION & SCOPES
-- ============================================================================

-- Define user scope for authentication
DEFINE SCOPE user_scope SESSION 24h
    SIGNIN (
        SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(password_hash, $password)
    )
    SIGNUP (
        CREATE user CONTENT {
            username: $username,
            email: $email,
            password_hash: crypto::argon2::generate($password),
            roles: ["user"],
            status: "active",
            created_at: time::now(),
            updated_at: time::now(),
            mfa_enabled: false
        }
    );

-- Define API scope for business clients
DEFINE SCOPE api_scope SESSION 720h  -- 30 days
    SIGNIN (
        SELECT * FROM api_key WHERE key_hash = crypto::sha256($api_key) AND status = "active"
    );

-- Define admin token
DEFINE TOKEN admin_token ON SCOPE user_scope TYPE HS512 VALUE $env::JWT_SECRET;

-- ============================================================================
-- FULL-TEXT SEARCH ANALYZERS & INDEXES
-- ============================================================================

-- Define custom analyzer for reports
DEFINE ANALYZER report_analyzer TOKENIZERS class, blank FILTERS lowercase, snowball(english), ascii;

-- Define analyzer for names (exact matching important)
DEFINE ANALYZER name_analyzer TOKENIZERS class FILTERS lowercase, ascii;

-- ============================================================================
-- USERS & AUTHENTICATION (with RLAC)
-- ============================================================================

DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select WHERE id = $auth.id OR $auth.roles CONTAINS "admin"
        FOR update WHERE id = $auth.id OR $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD username ON user TYPE string ASSERT string::len($value) > 2;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD roles ON user TYPE array<string> DEFAULT ["user"];
DEFINE FIELD status ON user TYPE string DEFAULT "active" ASSERT $value IN ["active", "suspended", "deleted"];
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD last_login ON user TYPE option<datetime>;
DEFINE FIELD mfa_enabled ON user TYPE bool DEFAULT false;
DEFINE FIELD mfa_secret ON user TYPE option<string>;

DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;
DEFINE INDEX idx_user_username ON user FIELDS username UNIQUE;

-- ============================================================================
-- REPORTS (with RLAC and Full-Text Search)
-- ============================================================================

DEFINE TABLE report SCHEMAFULL
    PERMISSIONS
        -- Public reports visible to all
        -- Submitted reports visible to submitter and reviewers
        FOR select WHERE
            visibility_tier = "public" OR
            submitted_by = $auth.id OR
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.id != NONE
        FOR update WHERE
            submitted_by = $auth.id OR
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD title ON report TYPE string;
DEFINE FIELD description ON report TYPE string;
DEFINE FIELD category ON report TYPE string;
DEFINE FIELD incident_date ON report TYPE option<datetime>;
DEFINE FIELD incident_location ON report TYPE option<geometry<point>>;
DEFINE FIELD street_name ON report TYPE option<string>;
DEFINE FIELD city ON report TYPE option<string>;
DEFINE FIELD postcode_district ON report TYPE option<string>;
DEFINE FIELD status ON report TYPE string DEFAULT "draft";
DEFINE FIELD visibility_tier ON report TYPE string DEFAULT "reviewers";
DEFINE FIELD harm_risk ON report TYPE string DEFAULT "low";
DEFINE FIELD source_refs ON report TYPE array<string> DEFAULT [];
DEFINE FIELD submitted_by ON report TYPE option<record<user>>;
DEFINE FIELD submitted_at ON report TYPE option<datetime>;
DEFINE FIELD created_at ON report TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON report TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_report_status ON report FIELDS status, created_at;
DEFINE INDEX idx_report_submitter ON report FIELDS submitted_by;
DEFINE INDEX idx_report_location ON report FIELDS incident_location;

-- Full-text search indexes
DEFINE INDEX report_title_idx ON report FIELDS title SEARCH ANALYZER report_analyzer BM25(1.2, 0.75) HIGHLIGHTS;
DEFINE INDEX report_description_idx ON report FIELDS description SEARCH ANALYZER report_analyzer BM25(1.2, 0.75) HIGHLIGHTS;

-- ============================================================================
-- CONVICTION RECORDS (with Full-Text Search and RLAC)
-- ============================================================================

DEFINE TABLE conviction_record SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            verification_status = "verified" OR
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.roles CONTAINS "user"
        FOR update WHERE $auth.roles CONTAINS "reviewer"
        FOR delete WHERE $auth.roles CONTAINS "admin";

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
DEFINE FIELD notes ON conviction_record TYPE string DEFAULT "";
DEFINE FIELD verification_status ON conviction_record TYPE string DEFAULT "unverified";
DEFINE FIELD verification_notes ON conviction_record TYPE option<string>;
DEFINE FIELD verified_by ON conviction_record TYPE option<record<user>>;
DEFINE FIELD verified_at ON conviction_record TYPE option<datetime>;
DEFINE FIELD created_at ON conviction_record TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON conviction_record TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_conviction_verification ON conviction_record FIELDS verification_status;
DEFINE INDEX conviction_name_idx ON conviction_record FIELDS full_name SEARCH ANALYZER name_analyzer BM25 HIGHLIGHTS;
DEFINE INDEX conviction_offense_idx ON conviction_record FIELDS offense_type SEARCH ANALYZER report_analyzer BM25;

-- ============================================================================
-- FACE DATASET WITH SURREALDB ML
-- ============================================================================

-- Define ML model for face embeddings
DEFINE ML face_embedding_model<0.1.0> FROM 'ml://model/face-recognition';

DEFINE TABLE face_dataset_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD linked_conviction_id ON face_dataset_item TYPE option<record<conviction_record>>;
DEFINE FIELD image_url ON face_dataset_item TYPE string;
DEFINE FIELD embedding ON face_dataset_item TYPE array<float>;  -- Computed by SurrealDB ML
DEFINE FIELD source_type ON face_dataset_item TYPE string;
DEFINE FIELD consent_obtained ON face_dataset_item TYPE bool;
DEFINE FIELD added_by ON face_dataset_item TYPE record<user>;
DEFINE FIELD added_at ON face_dataset_item TYPE datetime DEFAULT time::now();
DEFINE FIELD verified ON face_dataset_item TYPE bool DEFAULT false;

DEFINE INDEX idx_face_dataset_conviction ON face_dataset_item FIELDS linked_conviction_id;

-- ============================================================================
-- FACE SEARCH AUDIT (NO BIOMETRICS STORED)
-- ============================================================================

DEFINE TABLE face_search_audit SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            actor_id = $auth.id OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.id != NONE;

DEFINE FIELD actor_id ON face_search_audit TYPE option<record<user>>;
DEFINE FIELD actor_ip_hash ON face_search_audit TYPE string;
DEFINE FIELD timestamp ON face_search_audit TYPE datetime DEFAULT time::now();
DEFINE FIELD result_count ON face_search_audit TYPE int;
DEFINE FIELD confidence_bucket ON face_search_audit TYPE string;
DEFINE FIELD purpose ON face_search_audit TYPE option<string>;

DEFINE INDEX idx_face_search_time ON face_search_audit FIELDS timestamp;
DEFINE INDEX idx_face_search_actor ON face_search_audit FIELDS actor_id, timestamp;

-- ============================================================================
-- MISSING PERSON ALERTS (with RLAC and Geo)
-- ============================================================================

DEFINE TABLE missing_person_alert SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            status IN ["active", "verified"] OR
            created_by = $auth.id OR
            $auth.roles CONTAINS "reviewer"
        FOR create WHERE $auth.roles CONTAINS "user"
        FOR update WHERE
            created_by = $auth.id OR
            $auth.roles CONTAINS "reviewer"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD alert_id ON missing_person_alert TYPE string;
DEFINE FIELD full_name ON missing_person_alert TYPE string;
DEFINE FIELD age ON missing_person_alert TYPE int;
DEFINE FIELD description ON missing_person_alert TYPE string;
DEFINE FIELD last_seen_location ON missing_person_alert TYPE option<geometry<point>>;
DEFINE FIELD last_seen_date ON missing_person_alert TYPE datetime;
DEFINE FIELD photo_url ON missing_person_alert TYPE option<string>;
DEFINE FIELD contact_info ON missing_person_alert TYPE string;
DEFINE FIELD status ON missing_person_alert TYPE string DEFAULT "draft";
DEFINE FIELD priority ON missing_person_alert TYPE string DEFAULT "medium";
DEFINE FIELD geofence_radius_km ON missing_person_alert TYPE option<float>;
DEFINE FIELD active_until ON missing_person_alert TYPE datetime;
DEFINE FIELD created_by ON missing_person_alert TYPE record<user>;
DEFINE FIELD created_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD resolved_at ON missing_person_alert TYPE option<datetime>;

DEFINE INDEX idx_alert_status ON missing_person_alert FIELDS status, active_until;
DEFINE INDEX idx_alert_location ON missing_person_alert FIELDS last_seen_location;

-- ============================================================================
-- BUSINESS API (with RLAC)
-- ============================================================================

DEFINE TABLE business_tenant SCHEMAFULL
    PERMISSIONS
        FOR select WHERE id = $auth.tenant_id OR $auth.roles CONTAINS "admin"
        FOR update WHERE id = $auth.tenant_id OR $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD name ON business_tenant TYPE string;
DEFINE FIELD contact_email ON business_tenant TYPE string;
DEFINE FIELD industry ON business_tenant TYPE string;
DEFINE FIELD purpose_statement ON business_tenant TYPE string;
DEFINE FIELD status ON business_tenant TYPE string DEFAULT "pending";
DEFINE FIELD approved_by ON business_tenant TYPE option<record<user>>;
DEFINE FIELD approved_at ON business_tenant TYPE option<datetime>;
DEFINE FIELD rate_limit_tier ON business_tenant TYPE string DEFAULT "basic";
DEFINE FIELD created_at ON business_tenant TYPE datetime DEFAULT time::now();

DEFINE TABLE api_key SCHEMAFULL
    PERMISSIONS
        FOR select WHERE tenant_id = $auth.tenant_id OR $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.roles CONTAINS "admin"
        FOR delete WHERE tenant_id = $auth.tenant_id OR $auth.roles CONTAINS "admin";

DEFINE FIELD tenant_id ON api_key TYPE record<business_tenant>;
DEFINE FIELD key_hash ON api_key TYPE string;
DEFINE FIELD key_prefix ON api_key TYPE string;
DEFINE FIELD scopes ON api_key TYPE array<string> DEFAULT ["validate"];
DEFINE FIELD rate_limit_per_hour ON api_key TYPE int DEFAULT 100;
DEFINE FIELD status ON api_key TYPE string DEFAULT "active";
DEFINE FIELD last_used_at ON api_key TYPE option<datetime>;
DEFINE FIELD created_at ON api_key TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON api_key TYPE option<datetime>;

DEFINE INDEX idx_api_key_hash ON api_key FIELDS key_hash UNIQUE;

-- ============================================================================
-- GRAPH RELATIONSHIPS
-- ============================================================================

-- User reported a report
DEFINE TABLE reported_by SCHEMAFULL TYPE RELATION IN report OUT user
    PERMISSIONS
        FOR select WHERE $auth.roles CONTAINS "reviewer" OR $auth.roles CONTAINS "admin";

-- Report involves a person (conviction record)
DEFINE TABLE involves_person SCHEMAFULL TYPE RELATION IN report OUT conviction_record
    PERMISSIONS
        FOR select WHERE $auth.roles CONTAINS "reviewer" OR $auth.roles CONTAINS "admin";

-- Report located at a map entry
DEFINE TABLE located_at SCHEMAFULL TYPE RELATION IN report OUT map_entry
    PERMISSIONS FOR select WHERE true;

-- Person known in multiple locations
DEFINE TABLE known_in_area SCHEMAFULL TYPE RELATION IN conviction_record OUT map_entry
    PERMISSIONS FOR select WHERE $auth.roles CONTAINS "reviewer";

-- ============================================================================
-- SURREALDB FUNCTIONS
-- ============================================================================

-- Function: Ephemeral face search (query image NEVER stored)
DEFINE FUNCTION fn::search_faces($query_image: string, $actor_id: record<user>) {
    -- Compute embedding in memory only
    LET $query_embedding = ml::embedding::compute('face_embedding_model', $query_image);

    -- Search for similar faces
    LET $candidates = SELECT
        *,
        ml::similarity::cosine(embedding, $query_embedding) AS confidence
    FROM face_dataset_item
    WHERE ml::similarity::cosine(embedding, $query_embedding) > 0.75
    ORDER BY confidence DESC
    LIMIT 20;

    -- Determine confidence bucket
    LET $confidence_bucket = IF array::len($candidates) > 0 AND $candidates[0].confidence > 0.9 THEN
        "high"
    ELSE IF array::len($candidates) > 0 AND $candidates[0].confidence > 0.8 THEN
        "medium"
    ELSE
        "low"
    END;

    -- Create audit log (NO BIOMETRICS)
    CREATE face_search_audit CONTENT {
        actor_id: $actor_id,
        actor_ip_hash: crypto::sha256($auth.ip),
        timestamp: time::now(),
        result_count: array::len($candidates),
        confidence_bucket: $confidence_bucket,
        purpose: "investigation"
    };

    -- Return candidates (query embedding discarded after this)
    RETURN {
        candidates: $candidates,
        disclaimer: "These are potential matches requiring human verification. This is not a determination.",
        confidence_bucket: $confidence_bucket
    };
};

-- Function: Full-text search across reports
DEFINE FUNCTION fn::search_reports($query: string, $limit: int) {
    RETURN SELECT
        *,
        (search::score(1) * 2 + search::score(2)) AS relevance,
        search::highlight('<mark>', '</mark>', 1) AS title_highlight,
        search::highlight('<mark>', '</mark>', 2) AS description_highlight
    FROM report
    WHERE
        title @1@ $query OR
        description @2@ $query
    ORDER BY relevance DESC
    LIMIT $limit;
};

-- Function: Find connected reports (same person, location, or timeframe)
DEFINE FUNCTION fn::find_connected_reports($report_id: record<report>) {
    LET $report = SELECT * FROM $report_id;
    LET $people = $report->involves_person->conviction_record;
    LET $location = $report->located_at->map_entry;

    RETURN SELECT
        report.*,
        array::len(array::intersect(
            report->involves_person->conviction_record,
            $people
        )) AS common_people_count,
        geo::distance(
            report->located_at->map_entry[0].geometry,
            $location[0].geometry
        ) AS distance_meters,
        time::diff($report.incident_date, report.incident_date) AS time_diff_days
    FROM report
    WHERE
        report.id != $report_id AND
        (
            -- Same person involved
            array::len(array::intersect(
                report->involves_person->conviction_record,
                $people
            )) > 0 OR
            -- Nearby location (within 5km)
            geo::distance(
                report->located_at->map_entry[0].geometry,
                $location[0].geometry
            ) < 5000
        )
    ORDER BY (common_people_count * 10) DESC
    LIMIT 50;
};

-- Function: Auto-expire old alerts
DEFINE FUNCTION fn::expire_old_alerts() {
    UPDATE missing_person_alert
    SET status = "expired", updated_at = time::now()
    WHERE active_until < time::now() AND status = "active"
    RETURN AFTER;
};

-- Function: Calculate review priority score
DEFINE FUNCTION fn::calculate_priority($harm_risk: string, $created_at: datetime) {
    LET $risk_score = IF $harm_risk = "high" THEN 10
        ELSE IF $harm_risk = "medium" THEN 5
        ELSE 2 END;

    LET $age_hours = time::diff(time::now(), $created_at) / 3600;
    LET $age_score = math::min($age_hours / 24, 10);  -- Max 10 points for age

    RETURN $risk_score + $age_score;
};

-- Function: Get review queue with priority
DEFINE FUNCTION fn::get_review_queue($reviewer_id: record<user>, $limit: int) {
    RETURN SELECT
        *,
        fn::calculate_priority(harm_risk, created_at) AS priority_score
    FROM report
    WHERE status = "submitted"
    ORDER BY priority_score DESC
    LIMIT $limit;
};

-- Function: Find people with multiple reports in same area
DEFINE FUNCTION fn::find_patterns_multiple_reports_same_person() {
    RETURN SELECT
        conviction_record.full_name AS person,
        array::len(<-involves_person<-report) AS report_count,
        array::distinct(<-involves_person<-report.city) AS cities,
        array::min(<-involves_person<-report.created_at) AS first_report,
        array::max(<-involves_person<-report.created_at) AS latest_report
    FROM conviction_record
    WHERE array::len(<-involves_person<-report) > 2
    ORDER BY report_count DESC
    LIMIT 100;
};

-- ============================================================================
-- EVENTS (Automated Tasks)
-- ============================================================================

-- Event: Auto-expire alerts every hour
DEFINE EVENT expire_alerts_event ON TABLE missing_person_alert WHEN $event = "CREATE" OR $event = "UPDATE" THEN {
    fn::expire_old_alerts()
};

-- Event: Update report search index on change
DEFINE EVENT update_report_search ON TABLE report WHEN $event = "UPDATE" THEN {
    -- SurrealDB automatically updates full-text indexes
};

-- ============================================================================
-- MAP ENTRIES (with Geo and RLAC)
-- ============================================================================

DEFINE TABLE map_entry SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            visibility_tier = "public" OR
            $auth.roles CONTAINS "reviewer" OR
            harm_risk = "low"
        FOR create WHERE $auth.roles CONTAINS "reviewer"
        FOR update WHERE $auth.roles CONTAINS "reviewer"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD geometry ON map_entry TYPE geometry<point>;
DEFINE FIELD precision_class ON map_entry TYPE string;
DEFINE FIELD display_policy ON map_entry TYPE string DEFAULT "standard";
DEFINE FIELD linked_item_id ON map_entry TYPE option<record<publishable_item>>;
DEFINE FIELD linked_conviction_id ON map_entry TYPE option<record<conviction_record>>;
DEFINE FIELD street_name ON map_entry TYPE string;
DEFINE FIELD city ON map_entry TYPE string;
DEFINE FIELD postcode_district ON map_entry TYPE option<string>;
DEFINE FIELD visibility_tier ON map_entry TYPE string DEFAULT "public";
DEFINE FIELD harm_risk ON map_entry TYPE string DEFAULT "low";
DEFINE FIELD verified ON map_entry TYPE bool DEFAULT false;
DEFINE FIELD created_at ON map_entry TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_map_geometry ON map_entry FIELDS geometry;
DEFINE INDEX idx_map_visibility ON map_entry FIELDS visibility_tier, harm_risk;

-- ============================================================================
-- EVIDENCE & MEDIA ASSETS (with RLAC)
-- ============================================================================

DEFINE TABLE evidence SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.id != NONE
        FOR update WHERE
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD report_id ON evidence TYPE record<report>;
DEFINE FIELD media_asset_id ON evidence TYPE option<record<media_asset>>;
DEFINE FIELD evidence_type ON evidence TYPE string;  -- video, image, document, testimony, other
DEFINE FIELD description ON evidence TYPE string;
DEFINE FIELD collected_at ON evidence TYPE option<datetime>;
DEFINE FIELD chain_of_custody ON evidence TYPE array DEFAULT [];
DEFINE FIELD sealed ON evidence TYPE bool DEFAULT false;
DEFINE FIELD sealed_at ON evidence TYPE option<datetime>;
DEFINE FIELD sealed_by ON evidence TYPE option<record<user>>;
DEFINE FIELD submitted_by ON evidence TYPE option<record<user>>;
DEFINE FIELD created_at ON evidence TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON evidence TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_evidence_report ON evidence FIELDS report_id;
DEFINE INDEX idx_evidence_sealed ON evidence FIELDS sealed;

DEFINE TABLE media_asset SCHEMAFULL
    PERMISSIONS
        FOR select WHERE
            $auth.roles CONTAINS "reviewer" OR
            $auth.roles CONTAINS "admin"
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";

DEFINE FIELD filename ON media_asset TYPE string;
DEFINE FIELD mime_type ON media_asset TYPE string;
DEFINE FIELD file_size_bytes ON media_asset TYPE int;
DEFINE FIELD storage_path ON media_asset TYPE string;  -- Object storage path (S3, MinIO, etc.)
DEFINE FIELD storage_url ON media_asset TYPE option<string>;  -- Presigned URL for access
DEFINE FIELD thumbnail_url ON media_asset TYPE option<string>;
DEFINE FIELD preview_url ON media_asset TYPE option<string>;  -- For videos: preview clip
DEFINE FIELD duration_seconds ON media_asset TYPE option<float>;  -- For videos
DEFINE FIELD width ON media_asset TYPE option<int>;  -- For images/videos
DEFINE FIELD height ON media_asset TYPE option<int>;  -- For images/videos
DEFINE FIELD transcoding_status ON media_asset TYPE string DEFAULT "pending";  -- pending, processing, complete, failed
DEFINE FIELD checksum_sha256 ON media_asset TYPE string;
DEFINE FIELD uploaded_by ON media_asset TYPE option<record<user>>;
DEFINE FIELD uploaded_at ON media_asset TYPE datetime DEFAULT time::now();
DEFINE FIELD processed_at ON media_asset TYPE option<datetime>;

DEFINE INDEX idx_media_checksum ON media_asset FIELDS checksum_sha256;
DEFINE INDEX idx_media_status ON media_asset FIELDS transcoding_status;
