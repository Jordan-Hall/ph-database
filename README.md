# Predator Hunters Platform

A comprehensive public-interest journalism and safeguarding platform combining conviction tracking, video evidence management, missing-person alerts, and privacy-preserving face recognition.

## 🎯 Platform Overview

This platform provides:
- **Public-Interest Journalism**: Report submission, evidence handling, newsroom review workflows
- **Safeguarding**: Missing-person alerts with time-bound lifecycle
- **Street-Level Mapping**: OpenStreetMap-based visualization with anti-harassment controls
- **Video Evidence**: Upload, fast-review UX, thumbnails, and preview clips
- **Face Recognition**: Ephemeral, privacy-preserving search (never stores query images)
- **Business API**: Privacy-first validation service for risk assessment
- **Comprehensive Moderation**: Review workflows, corrections, takedowns, appeals

**⚠️ Built with Privacy & Safety First**
- No population-scale scanning or continuous monitoring
- Publication only for verified, lawful content
- Street-level mapping with precision controls
- Ephemeral face search (no query retention)
- Strong moderation and audit trails

---

## 🏗️ Architecture

### Technology Stack

**Frontend**
- Dioxus 0.7 (Rust → WebAssembly)
- MapLibre GL JS (OpenStreetMap vector tiles)
- UK GDS Design System

**Backend Services**
- Rust (Axum for services)
- SurrealDB 2.0+ (core database)
- FFmpeg (video processing)
- ONNX Runtime (face recognition)

**Infrastructure**
- MinIO (S3-compatible storage)
- TileServer GL (OSM tiles)
- Planetiler (tile generation)
- Nominatim (geocoding)
- Redis (rate limiting, sessions)

### Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│           Dioxus Web Apps (Public, Reviewer, Business)  │
└─────────────────────────────────────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────┐
│  API Gateway (Auth, RBAC, Rate Limits, Audit Logging)  │
└─────────────────────────────────────────────────────────┘
                            ▼
┌──────────────┬──────────────┬──────────────┬───────────┐
│  SurrealDB   │ Media Service│ AI Service   │ Alerts    │
│  (Core DB)   │ (Video)      │ (Face Search)│ Service   │
└──────────────┴──────────────┴──────────────┴───────────┘
                            ▼
┌──────────────┬──────────────┬────────────────────────────┐
│ MinIO (S3)   │ TileServer   │ Moderation Service        │
│ Object Store │ (OSM Tiles)  │ (Workflow Engine)         │
└──────────────┴──────────────┴────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

- Docker & Docker Compose
- 16GB RAM minimum (for video processing)
- 100GB+ disk space (for tiles + media)

### 1. Clone Repository

```bash
git clone https://github.com/Jordan-Hall/ph-database.git
cd ph-database
```

### 2. Configure Environment

```bash
cd deployment
cp .env.example .env
# Edit .env with your configuration
nano .env
```

**Critical Settings to Change:**
- `SURREAL_PASS`: Database password
- `JWT_SECRET`: Session signing key
- `MINIO_ROOT_PASSWORD`: Object storage password

### 3. Generate OSM Tiles

Download and generate tiles for your region:

```bash
# Download OSM data for UK
wget https://download.geofabrik.de/europe/united-kingdom-latest.osm.pbf

# Generate vector tiles with Planetiler
docker run -v $(pwd):/data ghcr.io/onthegomap/planetiler \
  --download --area=united-kingdom \
  --output=/data/tiles/united-kingdom.mbtiles
```

### 4. Initialize Database Schema

```bash
# Start SurrealDB
docker-compose up -d surrealdb

# Wait for DB to be ready
sleep 5

# Import schema
docker exec -i ph-surrealdb surreal import \
  --conn http://localhost:8000 \
  --user root --pass root \
  --ns prod --db main \
  /schemas/init.surql
```

### 5. Start All Services

```bash
docker-compose up -d
```

### 6. Access Applications

- **Public App**: http://localhost
- **API Gateway**: http://localhost:8080
- **MinIO Console**: http://localhost:9001
- **Tile Server**: http://localhost:8082

---

## 📚 Core Features

### 1. Report Submission & Evidence

**Public can submit:**
- Incident reports with location
- Video evidence (up to 500MB)
- Supporting documents
- Witness statements

**Automatic processing:**
- Virus scanning
- Video transcoding
- Thumbnail generation (every 5 seconds)
- Preview clip extraction (first 15 seconds)

**API Endpoints:**
```http
POST /api/v1/reports
POST /api/v1/reports/{id}/evidence
GET  /api/v1/reports/{id}
```

### 2. Video Fast-Review UI

For reviewers to quickly assess evidence:

**Features:**
- Auto-play preview clip
- Thumbnail timeline scrubber
- Speed controls (0.5x - 2x)
- Jump-to markers
- Flag key segments
- Side-by-side comparison

**Keyboard Shortcuts:**
- `Space`: Play/Pause
- `←/→`: Skip 5s
- `↑/↓`: Adjust speed
- `F`: Flag segment
- `A`: Approve

### 3. Street-Level Mapping (OpenStreetMap)

**Self-Hosted Stack:**
- Vector tiles via Planetiler
- TileServer GL for serving
- MapLibre GL JS in browser
- Optional Nominatim for geocoding

**Privacy Controls:**
- `precision_class`: street_segment | street_center | area
- `visibility_tier`: public | logged_in | verified_business | reviewers
- `harm_risk`: low | medium | high

**No exact addresses displayed as plain text.**

Map pins are:
- Snapped to street centerlines
- Placed at segment centroids
- Restricted by visibility tier

### 4. Missing Person Alerts

Amber-style alerts with lifecycle management:

**States:**
```
draft → verified → active → expired → resolved → archived
```

**Features:**
- Time-bound (TTL enforced)
- Priority levels (low, medium, high, critical)
- Optional geofencing
- Automatic expiry after TTL
- Archive after 90 days

**API Endpoints:**
```http
POST /api/v1/alerts
GET  /api/v1/alerts/active
GET  /api/v1/alerts/{id}
PUT  /api/v1/alerts/{id}/resolve
```

### 5. Face Recognition (Ephemeral & Privacy-Preserving)

**Strict Rules:**
1. Query images NEVER stored
2. Query embeddings NEVER persisted
3. Process in-memory only
4. Immediate purge after response
5. Always returns multiple candidates (or none)
6. Never a single "definitive match"

**Dataset:**
- Only lawful/consented images
- Court records, public records
- Verified by reviewers

**Response Format:**
```json
{
  "candidates": [
    {"id": "...", "confidence": 0.87, "requires_verification": true},
    {"id": "...", "confidence": 0.82, "requires_verification": true},
    {"id": "...", "confidence": 0.78, "requires_verification": true}
  ],
  "disclaimer": "Potential matches requiring human verification. Not a determination.",
  "confidence_bucket": "medium"
}
```

**Rate Limits:**
- 10 searches per hour per user
- Logged-in users only
- Full audit trail (no biometrics stored)

### 6. Business API (Privacy-First Validation)

For businesses to validate users without surveillance:

**Endpoint:**
```http
POST /api/v1/biz/validate
Authorization: Bearer {API_KEY}

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
  "decision_expires_at": "2024-04-01T00:00:00Z"
}
```

**What is NEVER returned:**
- Face images or embeddings
- Raw conviction records
- Personal addresses or names
- Bulk data exports

**Controls:**
- Rate limits (100-1000 req/hour depending on tier)
- IP allowlisting required
- Per-tenant audit logs
- Abuse detection

### 7. Moderation Workflows

**Review Queue:**
- Triage → Evidence Review → Source Validation → Publish Decision

**Actions:**
- Approve for publication
- Reject with reason
- Request more information
- Escalate to senior reviewer

**Corrections & Takedowns:**
- Public takedown request form
- Reviewer investigation
- Correction logging with audit trail
- Appeals process

---

## 🔐 Security & Compliance

### Authentication & Authorization

**User Auth:**
- Bcrypt password hashing (cost 12)
- JWT with 15-minute expiry
- Refresh tokens (30 days)
- MFA for reviewers and admins

**RBAC Matrix:**

| Role            | Reports | Review | Publish | Face Search | Admin |
|-----------------|---------|--------|---------|-------------|-------|
| public          | submit  | -      | -       | -           | -     |
| reporter        | submit  | -      | -       | -           | -     |
| reviewer        | view    | review | -       | yes         | -     |
| senior_reviewer | view    | review | publish | yes         | -     |
| admin           | all     | all    | all     | yes         | all   |

### Audit Logging

**All events logged:**
- Authentication attempts
- API requests
- Data access and modifications
- Face searches (no biometrics)
- Business API calls

**Retention:**
- Security events: 7 years
- Access logs: 2 years
- Face search: 1 year
- Sessions: 30 days

### Encryption

**In Transit:**
- TLS 1.3 everywhere
- HSTS enabled
- Certificate pinning

**At Rest:**
- Database encryption (SurrealDB native)
- S3 server-side encryption
- Secrets in Vault/KMS

---

## 📖 API Documentation

### Public Endpoints

```http
# Reports
POST   /api/v1/reports
GET    /api/v1/reports/{id}
POST   /api/v1/reports/{id}/evidence

# Map
GET    /api/v1/map/entries?bounds={...}

# Alerts
GET    /api/v1/alerts/active
GET    /api/v1/alerts/{id}

# Stories
GET    /api/v1/stories
POST   /api/v1/stories/submit

# Items (published content)
GET    /api/v1/items/{slug}
```

### Reviewer Endpoints

```http
# Review Queue
GET    /api/v1/review/queue
GET    /api/v1/review/{id}
POST   /api/v1/review/{id}/decision

# Publish
POST   /api/v1/publish/{item_id}
POST   /api/v1/takedown/{item_id}
POST   /api/v1/corrections/{item_id}
```

### Face Search

```http
POST   /api/v1/face-search
```

**Request:**
```json
{
  "image": "base64_encoded_image",
  "purpose": "investigation"
}
```

**Rate Limit:** 10/hour per user

### Business API

```http
POST   /api/v1/biz/validate
GET    /api/v1/biz/usage
POST   /api/v1/biz/keys
DELETE /api/v1/biz/keys/{id}
```

---

## 🗄️ Database Schema

See [`database/schemas/init.surql`](database/schemas/init.surql) for the complete SurrealDB schema.

**Core Tables:**
- `user`, `session`, `role_binding`
- `report`, `evidence`, `media_asset`
- `review_task`, `publishable_item`
- `map_entry`, `conviction_record`
- `missing_person_alert`, `survivor_story`
- `face_dataset_item`, `face_search_audit`
- `business_tenant`, `api_key`, `api_audit_log`
- `takedown_request`, `correction_log`

---

## 🛠️ Development

### Project Structure

```
ph-database/
├── src/                    # Dioxus web app (frontend)
│   ├── components/         # UI components
│   ├── pages/              # App pages
│   ├── services/           # API clients
│   └── main.rs
├── services/               # Backend microservices
│   ├── api-gateway/        # Main API + Auth
│   ├── media-service/      # Video processing
│   ├── ai-service/         # Face recognition
│   ├── alerts-service/     # Missing person alerts
│   └── moderation-service/ # Review workflows
├── database/
│   ├── schemas/            # SurrealDB schema
│   └── migrations/         # Schema migrations
├── deployment/
│   ├── docker-compose.yml  # Full stack deployment
│   ├── .env.example        # Configuration template
│   └── tileserver-config.json
├── tiles/                  # OSM tile data
├── models/                 # AI models (ONNX)
├── config/                 # Configuration files
└── TECHNICAL_PLAN.md       # Detailed architecture
```

### Running Services Locally

**Backend (Rust services):**
```bash
cd services/api-gateway
cargo run
```

**Frontend (Dioxus):**
```bash
dx serve
```

**Database:**
```bash
surreal start --user root --pass root file:data/db
```

### Running Tests

```bash
# All services
cargo test --workspace

# Specific service
cd services/api-gateway
cargo test
```

---

## 🚢 Deployment

### Production Deployment

1. **Configure Production Environment**
   ```bash
   cp deployment/.env.example deployment/.env
   # Edit with production values
   ```

2. **Generate SSL Certificates**
   ```bash
   certbot certonly --standalone -d yourdomain.com
   ```

3. **Deploy Stack**
   ```bash
   cd deployment
   docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

4. **Initialize Admin User**
   ```bash
   docker exec -it ph-api-gateway \
     /app/admin-cli create-user admin@example.com --role admin
   ```

### Scaling

**Horizontal Scaling:**
- API Gateway: Scale to N instances (stateless)
- Media Service: Scale with load balancer
- SurrealDB: Use clustering mode

**Vertical Scaling:**
- Media Service: More CPU for FFmpeg
- AI Service: More RAM for models

---

## 📊 Monitoring

### Metrics

**System Health:**
- Request rate, latency, errors
- Service uptime
- Database performance
- Queue depths

**Security:**
- Failed auth attempts
- Rate limit hits
- Suspicious patterns

**Business:**
- Report submissions
- Review queue depth
- Time-to-publish
- API usage by tenant

### Tools

- Prometheus + Grafana (metrics)
- Loki (log aggregation)
- Jaeger (distributed tracing)
- AlertManager (alerting)

---

## 🔒 Privacy & Legal

### GDPR Compliance

- ✅ Data minimisation by design
- ✅ Purpose limitation enforced
- ✅ Retention schedules automated
- ✅ Right to erasure implemented
- ✅ Data portability supported
- ✅ Privacy by default

### UK Data Protection Act 2018

- Lawful basis: Public interest journalism
- Special category data (biometrics) with explicit safeguards
- No automated decision-making without human oversight
- DPIA completed and documented

### Abuse Mitigation

| Subsystem | Risk | Mitigation |
|-----------|------|------------|
| Face Search | Mass surveillance | Rate limits, ephemeral processing, audit trail |
| Business API | Bulk enrichment | Minimal responses, IP allowlist, usage caps |
| Map Scraping | Automated harvesting | Rate limits, CAPTCHA, pagination limits |
| Alerts | Spam | Verification gate, max per region, abuse detection |

---

## 🤝 Contributing

This is a journalism and safeguarding platform. Contributions must:
- Maintain GDS design standards
- Preserve privacy protections
- Follow ethical usage guidelines
- Include tests and documentation

**Key Principles:**
1. Privacy by design
2. No surveillance capabilities
3. Strong moderation controls
4. Comprehensive audit trails
5. Ethical AI usage

---

## 📜 License

[Add your license here]

---

## 📞 Support

For issues or questions:
- GitHub Issues: https://github.com/Jordan-Hall/ph-database/issues
- Email: [Add contact email]

---

## ⚠️ Disclaimer

This platform is designed for legitimate journalism and safeguarding purposes only. Users are responsible for:

- Ensuring legal compliance in their jurisdiction
- Obtaining necessary consents
- Using data ethically and responsibly
- Not using for harassment, surveillance, or illegal purposes

The developers assume no liability for misuse of this software.

---

## 🗺️ Roadmap

### Phase 1: Foundation ✅
- SurrealDB schema
- API Gateway + Auth
- Basic reporting

### Phase 2: Media Pipeline (In Progress)
- Video upload + transcoding
- Thumbnail generation
- Fast-review UI

### Phase 3: Mapping
- OSM tile deployment
- MapLibre integration
- Precision controls

### Phase 4: Alerts
- Missing person alerts
- TTL lifecycle
- Geofencing

### Phase 5: Face Search
- Dataset management
- Ephemeral query processing
- Audit logging

### Phase 6: Business API
- Tenant management
- Validation endpoint
- Usage tracking

### Phase 7: Hardening
- Load testing
- Penetration testing
- Retention enforcement

---

## 📚 Additional Documentation

- [Technical Architecture Plan](TECHNICAL_PLAN.md)
- [API Reference](docs/API.md) _(coming soon)_
- [Deployment Guide](docs/DEPLOYMENT.md) _(coming soon)_
- [Security Controls](docs/SECURITY.md) _(coming soon)_
- [Privacy Policy](docs/PRIVACY.md) _(coming soon)_

---

Built with ❤️ for public-interest journalism and safeguarding.
