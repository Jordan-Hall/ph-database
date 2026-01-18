# Predator Hunters Platform - Technical Architecture Plan

## Executive Summary

A public-interest journalism and safeguarding platform combining:
- GDS-styled Dioxus 0.7 web clients
- SurrealDB core database
- Self-hosted OpenStreetMap rendering
- Privacy-preserving face recognition
- Video evidence with fast review
- Missing-person alerts
- Business API for validation
- Comprehensive moderation workflows

---

## 1. System Architecture

### 1.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                        CLIENTS                              │
├──────────────────┬──────────────────┬──────────────────────┤
│  Public App      │  Reviewer        │  Business Dashboard  │
│  (Dioxus 0.7)    │  Console         │  (Dioxus 0.7)       │
│                  │  (Dioxus 0.7)    │                      │
└──────────────────┴──────────────────┴──────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   API GATEWAY (Rust)                        │
│  Auth • RBAC • Rate Limiting • Audit • Request Validation   │
└─────────────────────────────────────────────────────────────┘
                           ▼
┌──────────────┬──────────────┬──────────────┬──────────────┐
│  SurrealDB   │ Media Service│ AI Service   │ Alerts       │
│  (Core DB)   │ (Video +     │ (Face Search)│ Service      │
│              │  Thumbnails) │              │              │
└──────────────┴──────────────┴──────────────┴──────────────┘
                           ▼
┌──────────────┬──────────────┬──────────────────────────────┐
│ Object Store │ Mapping Stack│ Moderation Service           │
│ (MinIO/S3)   │ (OSM Tiles)  │ (Workflow Engine)            │
└──────────────┴──────────────┴──────────────────────────────┘
```

### 1.2 Technology Stack

**Frontend**
- Dioxus 0.7 (Rust → WASM)
- MapLibre GL JS (OSM vector tiles)
- GDS Design System

**Backend Services**
- Rust (Axum/Actix for services)
- SurrealDB 2.0+
- FFmpeg (video processing)
- ONNX Runtime (face recognition)

**Infrastructure**
- MinIO (S3-compatible object storage)
- TileServer GL (OSM tile serving)
- Planetiler (tile generation)
- Nominatim (geocoding)
- Redis (rate limiting, sessions)

---

## 2. SurrealDB Data Model

### 2.1 Core Tables

```surql
-- Users and Authentication
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string ASSERT string::len($value) > 2;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD roles ON user TYPE array<string>;
DEFINE FIELD status ON user TYPE string ASSERT $value IN ['active', 'suspended', 'deleted'];
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD last_login ON user TYPE datetime;
DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;

-- Sessions
DEFINE TABLE session SCHEMAFULL;
DEFINE FIELD user_id ON session TYPE record<user>;
DEFINE FIELD token_hash ON session TYPE string;
DEFINE FIELD expires_at ON session TYPE datetime;
DEFINE FIELD ip_address ON session TYPE string;
DEFINE FIELD user_agent ON session TYPE string;
DEFINE FIELD created_at ON session TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_session_token ON session FIELDS token_hash UNIQUE;

-- Role Bindings
DEFINE TABLE role_binding SCHEMAFULL;
DEFINE FIELD user_id ON role_binding TYPE record<user>;
DEFINE FIELD role ON role_binding TYPE string;
DEFINE FIELD resource_type ON role_binding TYPE option<string>;
DEFINE FIELD resource_id ON role_binding TYPE option<string>;
DEFINE FIELD granted_by ON role_binding TYPE record<user>;
DEFINE FIELD granted_at ON role_binding TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON role_binding TYPE option<datetime>;

-- Reports (User Submissions)
DEFINE TABLE report SCHEMAFULL;
DEFINE FIELD title ON report TYPE string;
DEFINE FIELD description ON report TYPE string;
DEFINE FIELD category ON report TYPE string;
DEFINE FIELD incident_date ON report TYPE option<datetime>;
DEFINE FIELD incident_location ON report TYPE option<geometry<point>>;
DEFINE FIELD street_name ON report TYPE option<string>;
DEFINE FIELD city ON report TYPE option<string>;
DEFINE FIELD postcode_district ON report TYPE option<string>;
DEFINE FIELD status ON report TYPE string DEFAULT 'draft';
DEFINE FIELD visibility_tier ON report TYPE string DEFAULT 'reviewers';
DEFINE FIELD harm_risk ON report TYPE string DEFAULT 'low';
DEFINE FIELD source_refs ON report TYPE array<string>;
DEFINE FIELD submitted_by ON report TYPE option<record<user>>;
DEFINE FIELD submitted_at ON report TYPE option<datetime>;
DEFINE FIELD created_at ON report TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON report TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_report_status ON report FIELDS status;

-- Evidence (linked to reports)
DEFINE TABLE evidence SCHEMAFULL;
DEFINE FIELD report_id ON evidence TYPE record<report>;
DEFINE FIELD media_asset_id ON evidence TYPE option<record<media_asset>>;
DEFINE FIELD evidence_type ON evidence TYPE string;
DEFINE FIELD description ON evidence TYPE string;
DEFINE FIELD collected_at ON evidence TYPE option<datetime>;
DEFINE FIELD chain_of_custody ON evidence TYPE array<object>;
DEFINE FIELD sealed ON evidence TYPE bool DEFAULT false;
DEFINE FIELD sealed_at ON evidence TYPE option<datetime>;
DEFINE FIELD sealed_by ON evidence TYPE option<record<user>>;
DEFINE FIELD created_at ON evidence TYPE datetime DEFAULT time::now();

-- Media Assets (videos, images, documents)
DEFINE TABLE media_asset SCHEMAFULL;
DEFINE FIELD filename ON media_asset TYPE string;
DEFINE FIELD mime_type ON media_asset TYPE string;
DEFINE FIELD size_bytes ON media_asset TYPE int;
DEFINE FIELD storage_path ON media_asset TYPE string;
DEFINE FIELD storage_bucket ON media_asset TYPE string;
DEFINE FIELD checksum_sha256 ON media_asset TYPE string;
DEFINE FIELD variants ON media_asset TYPE object;
DEFINE FIELD thumbnail_path ON media_asset TYPE option<string>;
DEFINE FIELD preview_clip_path ON media_asset TYPE option<string>;
DEFINE FIELD duration_seconds ON media_asset TYPE option<float>;
DEFINE FIELD width ON media_asset TYPE option<int>;
DEFINE FIELD height ON media_asset TYPE option<int>;
DEFINE FIELD processing_status ON media_asset TYPE string DEFAULT 'pending';
DEFINE FIELD virus_scan_status ON media_asset TYPE string DEFAULT 'pending';
DEFINE FIELD uploaded_by ON media_asset TYPE record<user>;
DEFINE FIELD uploaded_at ON media_asset TYPE datetime DEFAULT time::now();

-- Review Tasks
DEFINE TABLE review_task SCHEMAFULL;
DEFINE FIELD report_id ON review_task TYPE record<report>;
DEFINE FIELD task_type ON review_task TYPE string;
DEFINE FIELD status ON review_task TYPE string DEFAULT 'pending';
DEFINE FIELD priority ON review_task TYPE int DEFAULT 5;
DEFINE FIELD assigned_to ON review_task TYPE option<record<user>>;
DEFINE FIELD assigned_at ON review_task TYPE option<datetime>;
DEFINE FIELD completed_at ON review_task TYPE option<datetime>;
DEFINE FIELD decision ON review_task TYPE option<string>;
DEFINE FIELD decision_notes ON review_task TYPE option<string>;
DEFINE FIELD created_at ON review_task TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_review_task_status ON review_task FIELDS status, priority;

-- Publishable Items (public pages)
DEFINE TABLE publishable_item SCHEMAFULL;
DEFINE FIELD slug ON publishable_item TYPE string;
DEFINE FIELD title ON publishable_item TYPE string;
DEFINE FIELD content_type ON publishable_item TYPE string;
DEFINE FIELD content ON publishable_item TYPE string;
DEFINE FIELD summary ON publishable_item TYPE option<string>;
DEFINE FIELD source_report_id ON publishable_item TYPE option<record<report>>;
DEFINE FIELD status ON publishable_item TYPE string DEFAULT 'draft';
DEFINE FIELD visibility_tier ON publishable_item TYPE string DEFAULT 'public';
DEFINE FIELD published_at ON publishable_item TYPE option<datetime>;
DEFINE FIELD published_by ON publishable_item TYPE option<record<user>>;
DEFINE FIELD corrections ON publishable_item TYPE array<object>;
DEFINE FIELD takedown_reason ON publishable_item TYPE option<string>;
DEFINE FIELD created_at ON publishable_item TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON publishable_item TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_item_slug ON publishable_item FIELDS slug UNIQUE;
DEFINE INDEX idx_item_status ON publishable_item FIELDS status, published_at;

-- Map Entries
DEFINE TABLE map_entry SCHEMAFULL;
DEFINE FIELD geometry ON map_entry TYPE geometry<point>;
DEFINE FIELD precision_class ON map_entry TYPE string;
DEFINE FIELD display_policy ON map_entry TYPE string;
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

-- Conviction Records
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
DEFINE FIELD notes ON conviction_record TYPE string;
DEFINE FIELD verification_status ON conviction_record TYPE string DEFAULT 'unverified';
DEFINE FIELD verification_notes ON conviction_record TYPE option<string>;
DEFINE FIELD verified_by ON conviction_record TYPE option<record<user>>;
DEFINE FIELD verified_at ON conviction_record TYPE option<datetime>;
DEFINE FIELD created_at ON conviction_record TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON conviction_record TYPE datetime DEFAULT time::now();

-- Missing Person Alerts
DEFINE TABLE missing_person_alert SCHEMAFULL;
DEFINE FIELD alert_id ON missing_person_alert TYPE string;
DEFINE FIELD full_name ON missing_person_alert TYPE string;
DEFINE FIELD age ON missing_person_alert TYPE int;
DEFINE FIELD description ON missing_person_alert TYPE string;
DEFINE FIELD last_seen_location ON missing_person_alert TYPE option<geometry<point>>;
DEFINE FIELD last_seen_date ON missing_person_alert TYPE datetime;
DEFINE FIELD photo_url ON missing_person_alert TYPE option<string>;
DEFINE FIELD contact_info ON missing_person_alert TYPE string;
DEFINE FIELD status ON missing_person_alert TYPE string DEFAULT 'draft';
DEFINE FIELD priority ON missing_person_alert TYPE string DEFAULT 'medium';
DEFINE FIELD geofence_radius_km ON missing_person_alert TYPE option<float>;
DEFINE FIELD active_until ON missing_person_alert TYPE datetime;
DEFINE FIELD created_by ON missing_person_alert TYPE record<user>;
DEFINE FIELD created_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON missing_person_alert TYPE datetime DEFAULT time::now();
DEFINE FIELD resolved_at ON missing_person_alert TYPE option<datetime>;
DEFINE FIELD archived_at ON missing_person_alert TYPE option<datetime>;
DEFINE INDEX idx_alert_status ON missing_person_alert FIELDS status, active_until;

-- Survivor Stories
DEFINE TABLE survivor_story SCHEMAFULL;
DEFINE FIELD title ON survivor_story TYPE string;
DEFINE FIELD content ON survivor_story TYPE string;
DEFINE FIELD author_pseudonym ON survivor_story TYPE option<string>;
DEFINE FIELD submitted_by ON survivor_story TYPE option<record<user>>;
DEFINE FIELD consent_given ON survivor_story TYPE bool DEFAULT false;
DEFINE FIELD consent_date ON survivor_story TYPE option<datetime>;
DEFINE FIELD status ON survivor_story TYPE string DEFAULT 'pending';
DEFINE FIELD published_at ON survivor_story TYPE option<datetime>;
DEFINE FIELD visibility ON survivor_story TYPE string DEFAULT 'private';
DEFINE FIELD created_at ON survivor_story TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON survivor_story TYPE datetime DEFAULT time::now();

-- Face Dataset (curated, lawful images only)
DEFINE TABLE face_dataset_item SCHEMAFULL;
DEFINE FIELD linked_conviction_id ON face_dataset_item TYPE option<record<conviction_record>>;
DEFINE FIELD image_url ON face_dataset_item TYPE string;
DEFINE FIELD embedding_ref ON face_dataset_item TYPE string;
DEFINE FIELD source_type ON face_dataset_item TYPE string;
DEFINE FIELD consent_obtained ON face_dataset_item TYPE bool;
DEFINE FIELD added_by ON face_dataset_item TYPE record<user>;
DEFINE FIELD added_at ON face_dataset_item TYPE datetime DEFAULT time::now();
DEFINE FIELD verified ON face_dataset_item TYPE bool DEFAULT false;

-- Face Search Audit (NO IMAGES, NO EMBEDDINGS STORED)
DEFINE TABLE face_search_audit SCHEMAFULL;
DEFINE FIELD actor_id ON face_search_audit TYPE option<record<user>>;
DEFINE FIELD actor_ip_hash ON face_search_audit TYPE string;
DEFINE FIELD timestamp ON face_search_audit TYPE datetime DEFAULT time::now();
DEFINE FIELD result_count ON face_search_audit TYPE int;
DEFINE FIELD confidence_bucket ON face_search_audit TYPE string;
DEFINE FIELD purpose ON face_search_audit TYPE option<string>;
DEFINE FIELD tenant_id ON face_search_audit TYPE option<record<business_tenant>>;
DEFINE INDEX idx_face_search_time ON face_search_audit FIELDS timestamp;

-- Business Tenants
DEFINE TABLE business_tenant SCHEMAFULL;
DEFINE FIELD name ON business_tenant TYPE string;
DEFINE FIELD contact_email ON business_tenant TYPE string;
DEFINE FIELD industry ON business_tenant TYPE string;
DEFINE FIELD purpose_statement ON business_tenant TYPE string;
DEFINE FIELD status ON business_tenant TYPE string DEFAULT 'pending';
DEFINE FIELD approved_by ON business_tenant TYPE option<record<user>>;
DEFINE FIELD approved_at ON business_tenant TYPE option<datetime>;
DEFINE FIELD rate_limit_tier ON business_tenant TYPE string DEFAULT 'basic';
DEFINE FIELD created_at ON business_tenant TYPE datetime DEFAULT time::now();

-- API Keys
DEFINE TABLE api_key SCHEMAFULL;
DEFINE FIELD tenant_id ON api_key TYPE record<business_tenant>;
DEFINE FIELD key_hash ON api_key TYPE string;
DEFINE FIELD key_prefix ON api_key TYPE string;
DEFINE FIELD scopes ON api_key TYPE array<string>;
DEFINE FIELD rate_limit_per_hour ON api_key TYPE int DEFAULT 100;
DEFINE FIELD allowlist_ips ON api_key TYPE array<string>;
DEFINE FIELD status ON api_key TYPE string DEFAULT 'active';
DEFINE FIELD last_used_at ON api_key TYPE option<datetime>;
DEFINE FIELD created_by ON api_key TYPE record<user>;
DEFINE FIELD created_at ON api_key TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON api_key TYPE option<datetime>;
DEFINE INDEX idx_api_key_hash ON api_key FIELDS key_hash UNIQUE;

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

-- Takedown Requests
DEFINE TABLE takedown_request SCHEMAFULL;
DEFINE FIELD item_id ON takedown_request TYPE record<publishable_item>;
DEFINE FIELD requested_by ON takedown_request TYPE option<record<user>>;
DEFINE FIELD requester_email ON takedown_request TYPE option<string>;
DEFINE FIELD reason ON takedown_request TYPE string;
DEFINE FIELD evidence ON takedown_request TYPE option<string>;
DEFINE FIELD status ON takedown_request TYPE string DEFAULT 'pending';
DEFINE FIELD reviewed_by ON takedown_request TYPE option<record<user>>;
DEFINE FIELD reviewed_at ON takedown_request TYPE option<datetime>;
DEFINE FIELD decision ON takedown_request TYPE option<string>;
DEFINE FIELD decision_notes ON takedown_request TYPE option<string>;
DEFINE FIELD created_at ON takedown_request TYPE datetime DEFAULT time::now();

-- Correction Log
DEFINE TABLE correction_log SCHEMAFULL;
DEFINE FIELD item_id ON correction_log TYPE record<publishable_item>;
DEFINE FIELD correction_type ON correction_log TYPE string;
DEFINE FIELD old_value ON correction_log TYPE option<string>;
DEFINE FIELD new_value ON correction_log TYPE option<string>;
DEFINE FIELD reason ON correction_log TYPE string;
DEFINE FIELD corrected_by ON correction_log TYPE record<user>;
DEFINE FIELD corrected_at ON correction_log TYPE datetime DEFAULT time::now();
```

### 2.2 Retention Policies

```surql
-- Automatic cleanup jobs (implement via scheduled tasks)

-- Session cleanup: 30 days inactive
DELETE session WHERE expires_at < time::now();

-- Missing person alerts: archive after expiry + 90 days
UPDATE missing_person_alert
SET status = 'archived', archived_at = time::now()
WHERE active_until < (time::now() - 90d) AND status != 'archived';

-- Face search audits: 1 year retention
DELETE face_search_audit WHERE timestamp < (time::now() - 365d);

-- API audit logs: 2 years retention
DELETE api_audit_log WHERE timestamp < (time::now() - 730d);
```

---

## 3. Service Architecture

### 3.1 API Gateway

**Responsibilities:**
- Authentication (JWT tokens, API keys)
- RBAC enforcement
- Rate limiting (per user, per IP, per API key)
- Request validation
- Audit logging
- CORS handling

**Stack:**
- Rust + Axum
- Redis for rate limiting
- JWT for sessions

**Rate Limits:**
- Public endpoints: 100 req/min/IP
- Authenticated: 1000 req/min/user
- Business API: configurable per tenant
- Face search: 10 req/hour/user

### 3.2 Media Service

**Responsibilities:**
- Upload handling (chunked, resumable)
- Virus scanning integration
- Video transcoding (FFmpeg)
- Thumbnail generation
- Preview clip extraction
- Signed URL generation

**Processing Pipeline:**
```
Upload → Virus Scan → Store Original → Transcode
                                    ├─→ Web variants (HLS/MP4)
                                    ├─→ Thumbnail strip
                                    └─→ Preview clip (10-20s)
```

**Stack:**
- Rust service
- FFmpeg for media processing
- MinIO/S3 for storage
- Background job queue

### 3.3 AI Service (Face Recognition)

**Strict Privacy Rules:**
1. Query images NEVER stored
2. Query embeddings NEVER persisted
3. Process in-memory only
4. Immediate purge after response
5. Always return multiple candidates (or none)
6. Never return single "definitive match"

**Flow:**
```
Query Upload → In-Memory Processing → Embedding →
Search Curated Index → Return Candidate Set → PURGE
```

**Response Format:**
```json
{
  "candidates": [
    {"id": "...", "confidence": 0.87, "requires_verification": true},
    {"id": "...", "confidence": 0.82, "requires_verification": true},
    {"id": "...", "confidence": 0.78, "requires_verification": true}
  ],
  "disclaimer": "These are potential matches requiring human verification. This is not a determination.",
  "confidence_bucket": "medium"
}
```

**Stack:**
- Rust + ONNX Runtime
- FaceNet or similar model
- Vector similarity search (FAISS or hnswlib)
- Strict memory management

### 3.4 Alerts Service

**Responsibilities:**
- Missing person alert lifecycle
- TTL enforcement
- Geofencing notifications
- Status transitions
- Automatic expiry

**Alert States:**
```
draft → verified → active → expired → archived
```

**Controls:**
- Max active alerts per region
- Abuse detection (excessive submissions)
- Verification gate before publishing

### 3.5 Moderation Service

**Workflow Engine:**
```
Report → Triage → Review Assignment → Decision → Publish/Reject
                                                ├─→ Corrections
                                                └─→ Appeals
```

**Review Types:**
- Evidence review
- Content verification
- Source validation
- Harm assessment
- Publication approval

**Automation:**
- Auto-assign by category
- Priority scoring
- SLA tracking
- Escalation rules

---

## 4. OpenStreetMap Self-Hosted Stack

### 4.1 Tile Generation

**Option 1: Planetiler (Recommended)**
```bash
# Generate vector tiles from OSM PBF
java -jar planetiler.jar \
  --download \
  --area=united-kingdom \
  --output=tiles.mbtiles
```

**Option 2: OpenMapTiles**
```bash
# Docker-based generation
docker-compose up
```

### 4.2 Tile Serving

**TileServer GL**
```bash
# Serve generated tiles
tileserver-gl tiles.mbtiles \
  --port 8080 \
  --public_url https://tiles.yourdomain.com
```

**CDN/Caching:**
- CloudFlare in front
- Nginx caching layer
- Prerendering for hot areas

### 4.3 Client Integration

**MapLibre GL JS in Dioxus:**
```rust
use dioxus::prelude::*;

#[component]
pub fn MapView() -> Element {
    rsx! {
        div { id: "map", style: "width: 100%; height: 600px;" }
        script {
            r#"
            maplibregl.accessToken = 'not-needed';
            const map = new maplibregl.Map({
                container: 'map',
                style: 'https://tiles.yourdomain.com/styles/basic/style.json',
                center: [-2.0, 54.5],
                zoom: 6
            });
            "#
        }
    }
}
```

### 4.4 Geocoding (Optional)

**Nominatim:**
```bash
# Docker deployment
docker run -d \
  --name nominatim \
  -p 8082:8080 \
  -e PBF_URL=https://download.geofabrik.de/europe/united-kingdom-latest.osm.pbf \
  mediagis/nominatim:4.2
```

---

## 5. Video Fast-Review UX

### 5.1 Processing Pipeline

**On Upload:**
1. Store original (raw)
2. Generate thumbnail strip (1 frame/5s)
3. Create preview clip (first 15s + motion detection)
4. Transcode for web (H.264/MP4, multiple qualities)
5. Extract audio waveform
6. Optional: Speech-to-text transcript

### 5.2 Reviewer Interface

**Features:**
- Thumbnail timeline scrubber
- Preview clip auto-play
- Speed controls (0.5x, 1x, 1.25x, 1.5x, 2x)
- Jump-to markers
- Flag segments as "key evidence"
- Side-by-side comparison mode
- Annotation tools

**Keyboard Shortcuts:**
- Space: Play/Pause
- ←/→: Skip 5s
- ↑/↓: Speed adjust
- F: Flag segment
- A: Approve
- R: Reject

---

## 6. Business API Design

### 6.1 Validate Endpoint

**Request:**
```http
POST /api/v1/biz/validate
Authorization: Bearer {API_KEY}
Content-Type: application/json

{
  "identifier": {
    "type": "document",
    "document_number": "AB123456",
    "dob": "1990-01-15"
  },
  "purpose": "employment_screening"
}
```

**Response:**
```json
{
  "request_id": "req_abc123",
  "status": "verified",
  "risk_level": "low",
  "reason_codes": [],
  "decision_expires_at": "2024-04-01T00:00:00Z",
  "disclaimer": "This is not a definitive determination. Further verification required."
}
```

**NEVER Returns:**
- Face images
- Embeddings
- Raw conviction records
- Personal addresses
- Names

### 6.2 Abuse Controls

**Rate Limits:**
- 100 requests/hour (basic tier)
- 1000 requests/hour (premium tier)
- Throttle on repeated failures

**Monitoring:**
- Alert on suspicious patterns
- Bulk lookup detection
- Unusual query volumes
- Geographic anomalies

**IP Allowlisting:**
- Required for production keys
- Maximum 5 IPs per tenant
- Validate on every request

---

## 7. Security Controls

### 7.1 Encryption

**In Transit:**
- TLS 1.3 everywhere
- HSTS enabled
- Certificate pinning for services

**At Rest:**
- Database encryption (SurrealDB native)
- Object storage encryption (S3 SSE)
- Secrets in Vault/KMS

### 7.2 Authentication & Authorization

**User Auth:**
- Bcrypt password hashing (cost 12)
- JWT with short expiry (15 min)
- Refresh tokens (30 days)
- MFA for reviewers and admins

**API Keys:**
- SHA-256 hashing
- Prefix for identification
- Scoped permissions
- Automatic rotation recommended

**RBAC Matrix:**
```
Role              | Reports | Review | Publish | API | Admin
------------------|---------|--------|---------|-----|------
public            | submit  | -      | -       | -   | -
reporter          | submit  | -      | -       | -   | -
reviewer          | view    | review | -       | -   | -
senior_reviewer   | view    | review | publish | -   | -
admin             | all     | all    | all     | all | all
business_api      | -       | -      | -       | api | -
```

### 7.3 Audit Logging

**All Events Logged:**
- Authentication attempts
- API requests
- Data access
- Modifications
- Permission changes
- Face searches
- Business API calls

**Log Structure:**
```json
{
  "timestamp": "2024-01-18T12:00:00Z",
  "event_type": "face_search",
  "actor_id": "user:abc123",
  "ip_address": "192.168.1.1",
  "action": "search",
  "resource": null,
  "outcome": "success",
  "metadata": {
    "result_count": 5,
    "confidence_bucket": "medium"
  }
}
```

**Retention:**
- Security events: 7 years
- Access logs: 2 years
- Face search: 1 year (no biometrics)
- Session logs: 30 days

---

## 8. Deployment Architecture

### 8.1 Container Stack

```yaml
# docker-compose.yml (simplified)
services:
  api-gateway:
    image: ph-api-gateway:latest
    ports: ["8080:8080"]
    environment:
      - DATABASE_URL=ws://surrealdb:8000
      - REDIS_URL=redis://redis:6379

  media-service:
    image: ph-media-service:latest
    volumes:
      - /tmp/uploads:/tmp/uploads

  ai-service:
    image: ph-ai-service:latest
    environment:
      - MODEL_PATH=/models/facenet.onnx

  surrealdb:
    image: surrealdb/surrealdb:latest
    command: start --user root --pass root file:data/db
    volumes:
      - surreal-data:/data

  redis:
    image: redis:7-alpine

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    volumes:
      - minio-data:/data

  tileserver:
    image: maptiler/tileserver-gl:latest
    volumes:
      - ./tiles:/data

volumes:
  surreal-data:
  minio-data:
```

### 8.2 Infrastructure Requirements

**Minimum Production:**
- API Gateway: 2x 2 CPU, 4GB RAM
- Media Service: 2x 4 CPU, 8GB RAM (for FFmpeg)
- AI Service: 1x 4 CPU, 16GB RAM (inference)
- SurrealDB: 1x 4 CPU, 16GB RAM
- Redis: 1x 1 CPU, 2GB RAM
- MinIO: 2x 2 CPU, 4GB RAM + storage
- TileServer: 1x 2 CPU, 4GB RAM

**Storage:**
- Database: 100GB SSD (grow as needed)
- Object Storage: 1TB+ (media accumulation)
- Tile Cache: 50GB SSD

---

## 9. Dioxus UI Architecture

### 9.1 Page Structure

```
/                       → Home / Search
/map                    → Interactive map view
/report                 → Submit report (multi-step)
/report/:id             → Report detail
/alerts                 → Active missing person alerts
/alerts/:id             → Alert detail
/stories                → Survivor stories (browse)
/stories/submit         → Submit story
/items/:slug            → Published item (public page)

/auth/login             → Login
/auth/register          → Register

/review                 → Reviewer dashboard
/review/queue           → Review queue
/review/:id             → Review detail + video fast-review
/review/:id/decision    → Decision form

/business               → Business dashboard
/business/keys          → API key management
/business/usage         → Usage statistics
/business/validate      → Validation endpoint tester

/admin                  → Admin dashboard
/admin/users            → User management
/admin/tenants          → Business tenant approvals
/admin/config           → System configuration
```

### 9.2 Component Architecture

```rust
// Shared components
mod components {
    pub mod header;
    pub mod nav;
    pub mod footer;
    pub mod map;
    pub mod video_player;
    pub mod evidence_viewer;
    pub mod modal;
    pub mod toast;
    pub mod pagination;
    pub mod search_bar;
}

// Page modules
mod pages {
    pub mod home;
    pub mod map;
    pub mod report;
    pub mod alerts;
    pub mod stories;
    pub mod items;
    pub mod auth;
    pub mod review;
    pub mod business;
    pub mod admin;
}

// Services (API clients)
mod services {
    pub mod api;
    pub mod auth;
    pub mod media;
    pub mod map_data;
}

// State management
mod state {
    pub mod auth_state;
    pub mod app_state;
}
```

---

## 10. Milestones & Implementation Plan

### Phase 1: Foundation (Weeks 1-3)
- [ ] SurrealDB schema deployment
- [ ] API Gateway with auth & RBAC
- [ ] Basic report submission
- [ ] User registration & login
- [ ] Audit logging infrastructure

### Phase 2: Media Pipeline (Weeks 4-6)
- [ ] Video upload (chunked)
- [ ] FFmpeg transcoding service
- [ ] Thumbnail & preview generation
- [ ] Object storage integration
- [ ] Basic evidence viewer

### Phase 3: Moderation (Weeks 7-9)
- [ ] Review queue system
- [ ] Workflow state machine
- [ ] Video fast-review UI
- [ ] Decision & approval flow
- [ ] Publish/reject/corrections

### Phase 4: Mapping (Weeks 10-12)
- [ ] OSM tile generation (Planetiler)
- [ ] TileServer GL deployment
- [ ] MapLibre integration
- [ ] Map entry CRUD
- [ ] Street-level precision controls
- [ ] Visibility tier enforcement

### Phase 5: Alerts (Weeks 13-14)
- [ ] Missing person alert schema
- [ ] Alert submission & verification
- [ ] TTL lifecycle management
- [ ] Geofencing (optional)
- [ ] Public alert display

### Phase 6: Face Search (Weeks 15-17)
- [ ] Face dataset management
- [ ] ONNX model integration
- [ ] Ephemeral query processing
- [ ] Multi-candidate results
- [ ] Audit logging (no biometrics)
- [ ] Rate limiting

### Phase 7: Business API (Weeks 18-20)
- [ ] Tenant management
- [ ] API key generation
- [ ] Validation endpoint
- [ ] Minimal response design
- [ ] Usage tracking & limits
- [ ] IP allowlisting

### Phase 8: Hardening (Weeks 21-24)
- [ ] Load testing
- [ ] Penetration testing
- [ ] Abuse simulation
- [ ] Retention enforcement
- [ ] Backup/restore procedures
- [ ] Monitoring & alerting
- [ ] Documentation

---

## 11. Abuse Mitigation Summary

| Subsystem | Abuse Risk | Mitigation |
|-----------|-----------|------------|
| **Face Search** | Mass surveillance | Rate limits (10/hr), logged-in only, audit trail, ephemeral processing |
| **Business API** | Bulk enrichment | No raw data export, minimal responses, IP allowlist, usage caps |
| **Map Scraping** | Automated harvesting | Rate limits, CAPTCHA, randomized responses, pagination limits |
| **Alert Spam** | Fake alerts | Verification gate, max alerts/region, abuse detection |
| **Video Uploads** | DoS, illegal content | Size limits, virus scan, queue throttling, review before publish |
| **Report Spam** | System flooding | CAPTCHA, rate limits, disposable email detection |
| **Street Mapping** | Harassment | No exact addresses, precision tiers, harm-risk flags, restricted visibility |

---

## 12. Compliance & Legal

### 12.1 GDPR Compliance

- Data minimisation by design
- Purpose limitation enforced
- Retention schedules automated
- Right to erasure implemented
- Data portability supported
- Privacy by default

### 12.2 UK Data Protection Act 2018

- Lawful basis documented (public interest journalism)
- Special category data (biometrics) processed only with explicit safeguards
- No profiling or automated decision-making without human oversight
- Data protection impact assessment (DPIA) completed

### 12.3 Rehabilitation of Offenders Act

- Spent convictions handling
- Automatic filtering after rehabilitation period
- Disclosure controls

---

## 13. Monitoring & Observability

### Metrics to Track

**System Health:**
- Request rate, latency, errors (RED)
- Service uptime
- Database performance
- Queue depths

**Security:**
- Failed auth attempts
- Rate limit hits
- Suspicious API patterns
- Face search frequency

**Business:**
- Report submissions
- Review queue depth
- Time-to-publish
- Alert lifecycle metrics
- API usage by tenant

**Tools:**
- Prometheus + Grafana
- Loki for logs
- Jaeger for tracing
- AlertManager

---

## 14. Future Enhancements

**Phase 9+:**
- Mobile apps (React Native or Flutter)
- Push notifications for alerts
- Advanced geospatial queries (PostGIS)
- Machine learning for content moderation
- Natural language processing for report triage
- Blockchain for audit trail immutability
- Federated identity (OAuth2/OIDC)
- Multi-language support
- Accessibility improvements (WCAG 2.1 AA)

---

## Conclusion

This architecture provides:
- ✅ Privacy-preserving face search with strict controls
- ✅ Business API that prevents surveillance abuse
- ✅ Street-level mapping with anti-harassment measures
- ✅ Comprehensive moderation workflows
- ✅ Video evidence with fast-review UX
- ✅ Missing-person alerts with TTL
- ✅ Self-hosted OSM (no Google dependency)
- ✅ Audit trails for all sensitive operations
- ✅ Scalable, secure, and compliant infrastructure

All subsystems include explicit abuse mitigations and comprehensive logging.
