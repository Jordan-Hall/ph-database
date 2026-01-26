# Predator Hunters Platform - Project Documentation

**Version:** 1.0.0
**Status:** Production Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [API Reference](#api-reference)
4. [Development Guide](#development-guide)
5. [Testing](#testing)
6. [License](#license)

---

## Overview

### Mission

A comprehensive public-interest journalism and safeguarding platform combining conviction tracking, video evidence management, missing-person alerts, and privacy-preserving face recognition.

### Core Capabilities

| Feature | Description |
|---------|-------------|
| **Public-Interest Journalism** | Report submission with evidence handling and newsroom review |
| **Safeguarding** | Missing-person alerts with time-bound lifecycle |
| **Street-Level Mapping** | OpenStreetMap visualization with anti-harassment controls |
| **Video Evidence** | Upload, processing, thumbnails, and fast-review UI |
| **Face Recognition** | Ephemeral, privacy-preserving search (never stores queries) |
| **Business API** | Privacy-first validation service for risk assessment |
| **Moderation** | Review workflows, corrections, takedowns, and appeals |
| **MFA** | TOTP-based multi-factor authentication with backup codes |

### Privacy & Safety First

- No population-scale scanning or continuous monitoring
- Publication only for verified, lawful content
- Street-level mapping with precision controls
- Ephemeral face search with no query retention
- Strong moderation workflows and audit trails
- Role-based access control with database-enforced permissions
- Rate limiting (100 requests/minute per IP)

---

## Features

### Authentication & MFA

**User Authentication:**
- Native SurrealDB SIGNUP/SIGNIN with Argon2 password hashing
- JWT tokens for session management (24h expiry)
- Role-based access: `user`, `reviewer`, `publisher`, `admin`
- Row-level permissions enforced at database level

**Multi-Factor Authentication:**
- TOTP (Time-based One-Time Password) with SHA1, 6 digits, 30s window
- QR code generation for authenticator apps
- 10 backup codes per user (8-character, bcrypt hashed)
- Login supports both TOTP and backup codes

### Reports Management

- Title, description, category, location
- Evidence upload (photos, videos, documents)
- Status workflow: draft → submitted → under_review → published/rejected
- Full-text search with BM25 ranking
- Graph queries for connection analysis
- Visibility tiers and harm risk assessment

### Review System

- Priority-based queue (harm_risk + age scoring)
- Task assignment to reviewers
- Decision workflow: approve, reject, request_changes
- Review escalation for stale items (>24h under review)
- Manual and automatic bulk escalation

### Publishing & Moderation

- Slug-based public URLs for published content
- Correction workflows with version history
- Takedown request handling
- Appeals process with automatic restoration on approval

### Missing Person Alerts

- Time-bound alerts with automatic expiration
- Priority levels: low, medium, high, critical
- Verification workflow before activation
- Resolution tracking with notes

### Map Features

- OpenStreetMap-based visualization
- Display policies: exact, street, postcode, town
- Privacy-preserving fuzzy display (~100m precision)
- Harm risk assessment: low, medium, high, critical
- Interactive markers with popups (UK GDS styled)

### Face Recognition

- Ephemeral processing (query images NEVER stored)
- Confidence thresholds and multi-candidate results
- Admin/reviewer access only
- Privacy-preserving audit logging

### Business API

- Three check types: Basic, Standard, Enhanced
- Confidence levels: none, low, medium, high, verified
- Tenant-based API key authentication
- Rate limiting tiers (100/500/2000 per hour)

### Survivor Stories

- Anonymous or attributed submission
- Consent tracking with timestamps
- Review workflow before publication
- Trigger warnings and pseudonyms

---

## API Reference

### Base URL

```
http://localhost:8080/api/v1
```

### Authentication

#### POST /auth/register

Register new user account.

**Request:**
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

**Response:**
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "user": {
    "id": "user:abc123",
    "username": "johndoe",
    "email": "john@example.com",
    "roles": ["user"]
  }
}
```

#### POST /auth/login

Login with email, password, and optional MFA code.

**Request (without MFA):**
```json
{
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

**Response (MFA required):**
```json
{
  "access_token": "",
  "refresh_token": "",
  "mfa_required": true
}
```

**Request (with MFA):**
```json
{
  "email": "john@example.com",
  "password": "SecurePass123!",
  "mfa_code": "123456"
}
```

#### POST /auth/mfa/setup

Generate MFA secret and QR code.

**Response:**
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code_url": "data:image/png;base64,iVBORw0KGgo...",
  "backup_codes": ["A1B2C3D4", "E5F6G7H8", "..."],
  "manual_entry_key": "JBSWY3DPEHPK3PXP"
}
```

#### POST /auth/mfa/enable

Enable MFA after verifying TOTP code.

**Request:**
```json
{
  "code": "123456"
}
```

#### POST /auth/refresh

Refresh access token.

**Request:**
```json
{
  "refresh_token": "eyJ..."
}
```

#### POST /auth/logout

Invalidate current session.

---

### Reports

#### POST /reports

Create new report. **Requires auth.**

**Request:**
```json
{
  "title": "Suspicious Activity",
  "description": "Detailed description...",
  "category": "suspicious_behavior",
  "location": "123 Main St, London",
  "latitude": 51.5074,
  "longitude": -0.1278
}
```

**Response:**
```json
{
  "id": "report:xyz789",
  "status": "submitted",
  "created_at": "2026-01-25T10:30:00Z"
}
```

#### GET /reports

List user's reports with pagination. **Requires auth.**

**Query Parameters:**
- `page` (default: 1)
- `per_page` (default: 20)

#### GET /reports/:id

Get report details. **Requires auth.**

#### GET /reports/search

Full-text search reports.

**Query Parameters:**
- `q` - Search query
- `limit` (default: 20)

#### GET /reports/:id/connections

Get connected reports via graph analysis. **Requires auth.**

---

### Review (Moderator/Admin)

#### GET /review/queue

Get review queue.

**Response:**
```json
{
  "reports": [
    {
      "id": "report:xyz789",
      "title": "Suspicious Activity",
      "status": "submitted",
      "created_at": "2026-01-25T10:30:00Z"
    }
  ],
  "count": 15
}
```

#### POST /review/:id/assign

Assign review to moderator.

#### POST /review/:id/decision

Make review decision.

**Request:**
```json
{
  "decision": "approve",
  "notes": "Verified and approved for publication"
}
```

#### POST /review/:id/escalate

Manually escalate review.

#### GET /review/stale

Find stale reviews (>24h).

#### POST /review/escalate-stale

Auto-escalate all stale reviews. **Admin only.**

---

### Publishing

#### POST /publish/report/:id

Publish approved report. **Moderator/Admin.**

**Response:**
```json
{
  "item_id": "publishable_item:pub123",
  "slug": "suspicious-activity-main-st-london"
}
```

#### GET /items/:slug

Get published item by slug. **Public.**

#### POST /publish/item/:id/correction

Add correction to published item.

**Request:**
```json
{
  "correction_type": "factual",
  "description": "Updated date information"
}
```

#### POST /publish/item/:id/takedown

Request takedown.

**Request:**
```json
{
  "reason": "legal",
  "evidence": "Court order reference..."
}
```

#### POST /publish/item/:id/appeal

Submit appeal for takedown.

**Request:**
```json
{
  "takedown_request_id": "takedown_request:td456",
  "appeal_reason": "Content was incorrectly removed...",
  "appellant_email": "john@example.com"
}
```

#### POST /publish/appeals/:id/review

Review appeal. **Admin only.**

**Request:**
```json
{
  "decision": "approve",
  "review_notes": "Appeal valid, restoring content"
}
```

---

### Alerts

#### GET /alerts/active

Get active missing person alerts. **Public.**

#### POST /alerts

Create new alert. **Requires auth.**

**Request:**
```json
{
  "full_name": "Jane Doe",
  "age": 25,
  "description": "Last seen wearing blue jacket...",
  "last_seen_location": "Central Park",
  "latitude": 40.7829,
  "longitude": -73.9654,
  "photo_url": "https://...",
  "contact_info": "Call 555-1234",
  "priority": "high",
  "expires_in_hours": 168
}
```

#### GET /alerts/:id

Get alert details.

#### PUT /alerts/:id/verify

Verify alert. **Reviewer/Admin.**

#### PUT /alerts/:id/resolve

Resolve alert with notes.

**Request:**
```json
{
  "resolution_notes": "Person found safe"
}
```

---

### Map

#### GET /map/entries

Get map entries within bounds.

**Query Parameters:**
- `min_lat`, `max_lat`, `min_lon`, `max_lon` - Bounding box
- `visibility_tier` - Filter by tier

#### POST /map/entries

Create map entry. **Requires auth.**

#### PUT /map/entries/:id/verify

Verify map entry. **Reviewer/Admin.**

---

### Stories

#### GET /stories

List published survivor stories. **Public.**

#### POST /stories/submit

Submit story (anonymous or authenticated).

**Request:**
```json
{
  "title": "My Story",
  "content": "Content...",
  "pseudonym": "Anonymous",
  "consent_given": true,
  "trigger_warnings": ["abuse"]
}
```

#### POST /stories/:id/review

Review story. **Reviewer/Admin.**

#### POST /stories/:id/publish

Publish approved story. **Reviewer/Admin.**

---

### Business API

**Authentication:** Include `X-API-Key` header.

#### POST /biz/validate

Validate individual.

**Request:**
```json
{
  "full_name": "John Doe",
  "date_of_birth": "1985-03-15",
  "check_type": "standard"
}
```

**Response:**
```json
{
  "match": true,
  "confidence": "high",
  "check_id": "check:chk999"
}
```

#### GET /biz/usage

Get API usage statistics.

#### POST /biz/tenants

Create business tenant. **Admin only.**

#### POST /biz/tenants/:id/keys

Generate API key for tenant. **Admin only.**

---

### Face Search

#### POST /face-search

Search faces. **Reviewer/Admin only.**

**Request:**
```json
{
  "image_base64": "iVBORw0KGgo...",
  "purpose": "investigation"
}
```

**Response:**
```json
{
  "candidates": [
    {"id": "...", "confidence": 0.87, "requires_verification": true},
    {"id": "...", "confidence": 0.82, "requires_verification": true}
  ],
  "disclaimer": "Potential matches requiring human verification.",
  "audit_id": "logged"
}
```

**Rate Limit:** 10 requests/hour per user.

---

### Admin

#### GET /admin/users

List all users with pagination. **Admin only.**

#### PATCH /admin/users/:id

Update user roles or status. **Admin only.**

**Request:**
```json
{
  "roles": ["user", "reviewer"],
  "status": "active"
}
```

#### GET /admin/audit-logs

Get audit logs with filters. **Admin only.**

**Query Parameters:**
- `action` - Filter by action type
- `resource_type` - Filter by resource
- `from`, `to` - Date range

---

## Development Guide

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Dioxus CLI
cargo install dioxus-cli

# Install SurrealDB CLI
curl -sSf https://install.surrealdb.com | sh
```

### Local Development

```bash
# Start backend services
cd deployment
docker-compose up -d

# Run API Gateway
cd services/api-gateway
cargo run

# Run Media Service
cd services/media-service
cargo run

# Run frontend (hot reload)
dx serve
```

### Project Structure

```
ph-database/
├── src/                    # Dioxus frontend
│   ├── api/                # API client
│   ├── components/         # UI components
│   ├── pages/              # Page components
│   ├── auth.rs             # Auth state
│   └── main.rs             # Entry point
├── services/
│   ├── api-gateway/        # Main API service
│   └── media-service/      # Video processing
├── database/
│   └── schemas/            # SurrealDB schema
├── deployment/
│   ├── docker-compose.yml
│   └── .env.example
└── assets/
    └── gds-styles.css      # UK GDS styling
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit
```

---

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### API Gateway Tests

```bash
cd services/api-gateway
cargo test --test integration_tests
```

**Test Coverage (13 tests):**
- Health check, user registration/login
- Protected routes, report operations
- Review queue, publishing workflow
- Business API, rate limiting
- Alerts lifecycle, map entries

### Media Service Tests

```bash
cd services/media-service
cargo test --test integration_tests
```

**Test Coverage (15 tests):**
- Health check, video upload
- Thumbnail generation, transcoding
- MinIO operations, error handling
- Concurrent uploads, cleanup

### Database Testing

```bash
# Connect to SurrealDB
surreal sql \
  --conn http://localhost:8000 \
  --user root --pass root \
  --ns development --db main

# Run test queries
SELECT * FROM report LIMIT 5;
```

---

## License

Copyright © 2026 Predator Hunters Platform. All rights reserved.

This software is provided for review and evaluation purposes. Commercial use, redistribution, or modification requires explicit written permission.

---

## Related Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture and SurrealDB native features
- [STATUS.md](STATUS.md) - Implementation progress and TODO items
- [DEPLOYMENT.md](DEPLOYMENT.md) - Production deployment guide
- [SECURITY.md](SECURITY.md) - Security controls and compliance
- [README.md](README.md) - Quick start guide

---

## Support

- **Issues:** https://github.com/Jordan-Hall/ph-database/issues
- **Documentation:** This repository
