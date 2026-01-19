# Predator Hunters Database - Production Deployment Guide

## Prerequisites

- Docker and Docker Compose installed
- At least 8GB RAM available
- 50GB+ disk space for database and media storage
- SSL certificates for HTTPS (recommended for production)

## Quick Start

### 1. Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your production values
nano .env
```

**CRITICAL**: Update these values in `.env`:
- `SURREAL_PASS` - Strong database password
- `JWT_SECRET` - Minimum 32 character random string
- `MINIO_ROOT_PASSWORD` - Strong MinIO password
- `GRAFANA_PASSWORD` - Strong Grafana password

### 2. Generate JWT Secret

```bash
# Generate a secure JWT secret
openssl rand -base64 32
```

### 3. Initialize Database Schemas

```bash
# Start only the database first
docker-compose up -d surrealdb redis

# Wait for database to be ready
sleep 10

# Apply database schemas
docker exec -i ph-surrealdb surreal import \
  --conn http://localhost:8000 \
  --user root \
  --pass "${SURREAL_PASS}" \
  --ns predator_hunters \
  --db main \
  /schemas/enhanced.surql
```

### 4. Start All Services

```bash
cd deployment

# Build and start all services
docker-compose up -d

# Check service health
docker-compose ps
```

### 5. Verify Deployment

```bash
# Check API Gateway health
curl http://localhost:8080/health

# Check Media Service health
curl http://localhost:8081/health

# Check Prometheus metrics
curl http://localhost:8080/metrics

# Access Grafana dashboard
# http://localhost:3000 (admin / your-password)
```

## Service Architecture

### Services Overview

| Service | Port | Purpose |
|---------|------|---------|
| API Gateway | 8080 | Main REST API |
| Media Service | 8081 | Video processing & storage |
| AI Service | 8082 | Face recognition |
| Alerts Service | 8083 | Missing person alerts |
| Moderation Service | 8084 | Review queue |
| Tileserver | 8082 | Map tiles |
| Nominatim | 8085 | Geocoding |
| SurrealDB | 8000 | Database |
| Redis | 6379 | Cache & sessions |
| MinIO | 9000 | Object storage |
| MinIO Console | 9001 | Storage admin UI |
| Prometheus | 9090 | Metrics |
| Grafana | 3000 | Dashboards |
| Loki | 3100 | Log aggregation |

### Network Architecture

All services communicate via the `ph-network` Docker bridge network. Services are isolated and only expose necessary ports to the host.

## Database Management

### Backup

```bash
# Export database backup
docker exec ph-surrealdb surreal export \
  --conn http://localhost:8000 \
  --user root \
  --pass "${SURREAL_PASS}" \
  --ns predator_hunters \
  --db main \
  /data/backup-$(date +%Y%m%d).surql

# Copy backup from container
docker cp ph-surrealdb:/data/backup-$(date +%Y%m%d).surql ./backups/
```

### Restore

```bash
# Copy backup to container
docker cp ./backups/backup.surql ph-surrealdb:/data/

# Import backup
docker exec -i ph-surrealdb surreal import \
  --conn http://localhost:8000 \
  --user root \
  --pass "${SURREAL_PASS}" \
  --ns predator_hunters \
  --db main \
  /data/backup.surql
```

## Storage Management

### MinIO Management

Access MinIO Console at `http://localhost:9001`

Default buckets created automatically:
- `evidence` - Evidence files (private)
- `media` - Public media files
- `thumbnails` - Generated thumbnails
- `previews` - Video preview clips

### Backup MinIO Data

```bash
# Backup all buckets
docker run --rm \
  --network deployment_ph-network \
  -v $(pwd)/minio-backup:/backup \
  minio/mc:latest \
  mirror myminio/media /backup/media

# Repeat for other buckets
```

## Monitoring

### Prometheus

Access Prometheus at `http://localhost:9090`

Key metrics to monitor:
- `http_requests_total` - Request count
- `http_request_duration_seconds` - Response times
- `process_cpu_usage` - CPU usage per service
- `process_memory_bytes` - Memory usage

### Grafana Dashboards

Access Grafana at `http://localhost:3000`

Default credentials: `admin / your-password` (from `.env`)

Pre-configured data sources:
- Prometheus (metrics)
- Loki (logs)

### Log Aggregation

```bash
# View logs in real-time
docker-compose logs -f

# View specific service logs
docker-compose logs -f api-gateway

# Query logs via Loki
curl -G http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={service="api-gateway"}'
```

## Security Hardening

### 1. Network Security

For production, update `docker-compose.yml` to:
- Remove port mappings for internal services
- Use nginx reverse proxy for SSL termination
- Implement rate limiting at proxy level

### 2. Database Security

```bash
# Create read-only user for reporting
docker exec -it ph-surrealdb surreal sql \
  --conn http://localhost:8000 \
  --user root \
  --pass "${SURREAL_PASS}" \
  --ns predator_hunters \
  --db main \
  "DEFINE USER analyst ON DATABASE PASSWORD 'secure-password' ROLES VIEWER;"
```

### 3. API Security

- Enable CORS with specific origins (update `docker-compose.yml`)
- Implement rate limiting per IP
- Use API keys for business API access
- Regular security audits

### 4. MinIO Security

```bash
# Create read-only policy for specific bucket
docker exec -it ph-minio mc admin policy create myminio readonly-media \
  /path/to/readonly-policy.json
```

## Scaling

### Horizontal Scaling

To scale individual services:

```bash
# Scale API Gateway to 3 instances
docker-compose up -d --scale api-gateway=3

# Scale Media Service to 2 instances
docker-compose up -d --scale media-service=2
```

Add a load balancer (nginx/traefik) in front of scaled services.

### Vertical Scaling

Update service resource limits in `docker-compose.yml`:

```yaml
services:
  api-gateway:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

## Troubleshooting

### Service Won't Start

```bash
# Check service logs
docker-compose logs api-gateway

# Check service status
docker-compose ps

# Restart specific service
docker-compose restart api-gateway
```

### Database Connection Issues

```bash
# Check database health
curl http://localhost:8000/health

# Test database connection
docker exec -it ph-surrealdb surreal sql \
  --conn http://localhost:8000 \
  --user root \
  --pass "${SURREAL_PASS}"
```

### Storage Issues

```bash
# Check MinIO health
curl http://localhost:9000/minio/health/live

# List buckets
docker exec ph-minio-client mc ls myminio/

# Check disk space
docker system df
```

### Performance Issues

```bash
# Check resource usage
docker stats

# Check for slow queries in logs
docker-compose logs api-gateway | grep "slow query"

# Monitor Prometheus metrics
curl http://localhost:8080/metrics | grep http_request_duration
```

## Maintenance

### Regular Tasks

1. **Daily**
   - Check service health
   - Review error logs
   - Monitor disk space

2. **Weekly**
   - Database backup
   - MinIO backup
   - Review security logs
   - Update dashboards

3. **Monthly**
   - Update Docker images
   - Security patches
   - Performance tuning
   - Capacity planning

### Updates

```bash
# Pull latest images
docker-compose pull

# Rebuild services
docker-compose build --no-cache

# Rolling update (zero downtime)
docker-compose up -d --no-deps --build api-gateway
```

## Disaster Recovery

### Full System Restore

1. Install Docker and Docker Compose
2. Clone repository
3. Copy `.env` file
4. Restore database backup
5. Restore MinIO data
6. Start all services
7. Verify functionality

### Recovery Time Objective (RTO)

Target: < 2 hours for full system restore

### Recovery Point Objective (RPO)

Target: < 24 hours data loss (daily backups)

## Support

For issues or questions:
- Check logs first: `docker-compose logs -f`
- Review Grafana dashboards for metrics
- Check GitHub issues
- Contact system administrator

## Production Checklist

- [ ] Strong passwords set in `.env`
- [ ] JWT secret generated and configured
- [ ] SSL certificates installed
- [ ] Firewall configured
- [ ] Database backups scheduled
- [ ] MinIO backups scheduled
- [ ] Monitoring configured
- [ ] Log aggregation working
- [ ] Alerting configured
- [ ] Security hardening applied
- [ ] Rate limiting configured
- [ ] CORS properly configured
- [ ] Documentation reviewed
- [ ] Disaster recovery plan tested
