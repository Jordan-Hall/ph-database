# Security Documentation

## Security Overview

The Predator Hunters Database platform implements multiple layers of security to protect sensitive data and ensure platform integrity.

## Authentication & Authorization

### JWT Token Security

- **Algorithm**: HS256 (HMAC with SHA-256)
- **Token Expiry**: Configurable (default 24 hours)
- **Refresh Strategy**: Client must re-authenticate after expiry
- **Secret Storage**: Environment variable only, never committed

**Best Practices**:
```bash
# Generate strong JWT secret (minimum 32 characters)
openssl rand -base64 32

# Set in .env
JWT_SECRET=your-generated-secret-here
JWT_EXPIRY_HOURS=24
```

### Password Security

- **Hashing Algorithm**: Argon2id (winner of Password Hashing Competition)
- **Salt**: Automatically generated per password
- **Cost Parameters**: Tuned for ~100ms computation time

**Implementation**:
```rust
// Password is hashed using Argon2id before storage
let password_hash = argon2::hash_encoded(
    password.as_bytes(),
    salt,
    &argon2::Config::default()
)?;
```

### Role-Based Access Control (RBAC)

User roles and permissions:

| Role | Permissions |
|------|-------------|
| `user` | Submit reports, view own submissions |
| `reviewer` | Review queue access, approve/reject reports |
| `publisher` | Publish approved reports, manage corrections |
| `admin` | Full system access, user management |

**Database-Level Enforcement**:
```sql
-- SurrealDB RLAC (Row-Level Access Control)
DEFINE TABLE report SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.id = submitted_by OR $auth.roles CONTAINS "reviewer"
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.roles CONTAINS "reviewer" OR $auth.roles CONTAINS "admin"
        FOR delete WHERE $auth.roles CONTAINS "admin";
```

## API Security

### Rate Limiting

**Implementation**: Redis-backed token bucket algorithm via `governor` crate

**Default Limits**:
- Anonymous: 10 requests/minute
- Authenticated: 60 requests/minute
- Business API Basic: 100 requests/hour
- Business API Standard: 500 requests/hour
- Business API Premium: 2000 requests/hour

**Configuration**:
```rust
// Adjust rate limits per environment
const RATE_LIMIT_ANONYMOUS: u32 = 10;
const RATE_LIMIT_AUTHENTICATED: u32 = 60;
```

### CORS Configuration

**Development** (permissive):
```yaml
CORS_ALLOWED_ORIGINS: "*"
CORS_ALLOWED_METHODS: "*"
CORS_ALLOWED_HEADERS: "*"
```

**Production** (restrictive):
```rust
let cors = CorsLayer::new()
    .allow_origin("https://yourdomain.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .max_age(Duration::from_secs(3600));
```

### Request Size Limits

- **Standard API requests**: 10MB
- **Media uploads**: 5GB (configurable via `MAX_VIDEO_SIZE_MB`)
- **JSON payload**: 1MB

```rust
// Apply size limits in axum
.layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
```

## Data Protection

### Personally Identifiable Information (PII)

**Redaction Strategy**:
- Full names stored in secure database
- Public display uses initials only (e.g., "John Smith" → "J.S.")
- Addresses stored with precision control
- DOB stored securely, only year displayed publicly

**Geo-Location Privacy**:
```rust
// Fuzzy display reduces precision to ~100m
pub enum DisplayPolicy {
    Standard,  // Full precision for reviewers
    Fuzzy,     // Rounded to 3 decimal places (public)
    Hidden,    // Not displayed at all
}
```

### Evidence Storage

**Media Files**:
- Stored in MinIO with private bucket access
- Presigned URLs for temporary access (24 hour expiry)
- Virus scanning before storage (TODO: implement ClamAV integration)
- Audit logging for all access

**Access Control**:
```rust
// Evidence bucket: private by default
// Public buckets: only for published, redacted content
mc anonymous set none myminio/evidence
mc anonymous set download myminio/media
```

### Encryption

**At Rest**:
- MinIO supports encryption at rest (enable in production)
- SurrealDB file-based encryption available
- Consider full disk encryption for production hosts

**In Transit**:
- **Development**: HTTP (Docker internal network)
- **Production**: HTTPS only (enforce via reverse proxy)

**Recommended nginx SSL configuration**:
```nginx
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers HIGH:!aNULL:!MD5;
ssl_prefer_server_ciphers on;
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;
```

## Database Security

### SurrealDB Hardening

**1. Authentication**:
```bash
# Use strong root password
SURREAL_PASS=$(openssl rand -base64 32)

# Create limited-privilege users
DEFINE USER api_user ON DATABASE PASSWORD 'secure-pass' ROLES EDITOR;
DEFINE USER analyst ON DATABASE PASSWORD 'secure-pass' ROLES VIEWER;
```

**2. Network Isolation**:
- Database not exposed to public internet
- Only accessible via Docker bridge network
- Consider firewall rules for additional protection

**3. Query Logging**:
```bash
# Enable query logging for audit
SURREAL_LOG=debug
```

### Redis Security

**Configuration**:
```bash
# Require password for Redis
redis-server --requirepass your-secure-password

# Disable dangerous commands
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command CONFIG ""
```

**Connection String**:
```bash
REDIS_URL=redis://:password@localhost:6379
```

## Input Validation

### Server-Side Validation

**All inputs validated using `validator` crate**:
```rust
#[derive(Deserialize, Validate)]
pub struct CreateReportRequest {
    #[validate(length(min = 2, max = 100))]
    pub subject_name: String,

    #[validate(email)]
    pub reporter_email: Option<String>,

    #[validate(length(min = 50, max = 10000))]
    pub description: String,

    #[validate(url)]
    pub evidence_url: Option<String>,
}
```

### SQL Injection Prevention

**SurrealDB Native Protection**:
- Parameterized queries prevent injection
- Input sanitization at application level
- No raw string concatenation in queries

```rust
// Safe: parameterized query
let result: Vec<Report> = db
    .query("SELECT * FROM report WHERE id = $id")
    .bind(("id", report_id))
    .await?;

// NEVER do this:
// let query = format!("SELECT * FROM report WHERE id = '{}'", report_id);
```

### XSS Prevention

**Content Security Policy**:
```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
```

**HTML Escaping**:
- All user-generated content escaped before display
- Markdown rendering sanitized
- No `dangerouslySetInnerHTML` equivalent

## Audit Logging

### Tracked Events

All sensitive operations logged:
- User authentication (login/logout)
- Report submissions
- Review decisions
- Publishing actions
- Corrections and takedowns
- Face recognition queries
- Business API validations
- Admin actions

**Log Format**:
```rust
pub struct AuditLog {
    pub event_type: String,      // "report_submitted", "report_approved", etc.
    pub actor_id: String,         // User performing action
    pub target_id: Option<String>, // Resource affected
    pub action: String,           // Description of action
    pub metadata: serde_json::Value, // Additional context
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

### Log Retention

- **Security logs**: 1 year minimum
- **Access logs**: 90 days
- **Error logs**: 30 days
- **Debug logs**: 7 days

### Log Analysis

Use Loki for centralized log aggregation:
```bash
# Query audit logs
curl -G http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={service="api-gateway",level="audit"}'
```

## Vulnerability Management

### Dependency Scanning

**Cargo Audit**:
```bash
# Install cargo-audit
cargo install cargo-audit

# Scan for vulnerabilities
cargo audit

# Update dependencies
cargo update
```

**Automated Scanning**:
```yaml
# .github/workflows/security.yml
name: Security Audit
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Docker Image Scanning

**Trivy Scanner**:
```bash
# Scan Docker images for vulnerabilities
trivy image api-gateway:latest
trivy image media-service:latest

# Scan for misconfigurations
trivy config deployment/
```

### Penetration Testing

**Recommended Schedule**:
- Internal testing: Quarterly
- External penetration test: Annually
- Bug bounty program: Consider for production

## Incident Response

### Response Plan

1. **Detection**: Monitor logs, metrics, alerts
2. **Containment**: Isolate affected services
3. **Investigation**: Analyze logs, identify root cause
4. **Remediation**: Apply fixes, update systems
5. **Recovery**: Restore services, verify functionality
6. **Lessons Learned**: Document incident, update procedures

### Emergency Contacts

```bash
# Store in secure location (e.g., password manager)
SECURITY_LEAD_EMAIL=security@example.com
SECURITY_LEAD_PHONE=+1-xxx-xxx-xxxx
ON_CALL_ROTATION=https://pagerduty.com/...
```

### Breach Notification

Follow GDPR requirements:
- Notify data protection authority within 72 hours
- Notify affected users without undue delay
- Document all breaches and responses

## Security Checklist

### Pre-Production

- [ ] Strong passwords set for all services
- [ ] JWT secret generated (min 32 chars)
- [ ] HTTPS enabled with valid certificates
- [ ] CORS restricted to specific origins
- [ ] Rate limiting configured
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention verified
- [ ] XSS prevention implemented
- [ ] CSRF protection enabled
- [ ] Security headers configured
- [ ] Audit logging enabled
- [ ] Error messages don't leak sensitive info
- [ ] Default credentials changed
- [ ] Unnecessary services disabled
- [ ] Firewall rules configured
- [ ] Backup encryption enabled
- [ ] Dependency scan clean
- [ ] Docker image scan clean
- [ ] Penetration test completed

### Production Monitoring

- [ ] Failed login attempts monitored
- [ ] Unusual API activity alerts configured
- [ ] Database query performance monitored
- [ ] Disk space alerts configured
- [ ] SSL certificate expiry alerts
- [ ] Log aggregation working
- [ ] Metrics dashboards reviewed daily
- [ ] Security patches applied monthly

## Reporting Security Issues

**DO NOT** open public GitHub issues for security vulnerabilities.

Instead:
1. Email: security@example.com
2. Include detailed description
3. Provide steps to reproduce
4. Expected disclosure timeline

We will:
- Acknowledge within 24 hours
- Provide initial assessment within 72 hours
- Work on fix and coordinate disclosure
- Credit security researchers (if desired)

## Security Updates

Subscribe to security advisories:
- Rust security advisory database: https://rustsec.org/
- Docker security announcements
- SurrealDB security releases
- Redis security announcements

## Compliance

### GDPR

- **Right to Access**: Users can export their data
- **Right to Erasure**: Users can request deletion (with legal review)
- **Right to Rectification**: Users can update their information
- **Data Minimization**: Only collect necessary data
- **Purpose Limitation**: Data used only for stated purposes
- **Privacy by Design**: Security built into architecture

### Data Retention

- **User accounts**: Retained while active, 90 days after deactivation
- **Reports**: Retained as long as legally necessary
- **Audit logs**: 1 year minimum
- **Media files**: Retained with associated reports
- **Backups**: 30 days retention

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [SurrealDB Security](https://surrealdb.com/docs/security)
