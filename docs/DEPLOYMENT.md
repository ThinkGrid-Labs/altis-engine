# Deployment Guide

## Overview

This guide covers deploying Altis Engine to production environments.

---

## Prerequisites

- Docker & Docker Compose (development)
- Kubernetes cluster (production)
- PostgreSQL 15+ (managed service recommended)
- Redis 7+ (managed service recommended)
- Kafka (managed service recommended)

---

## Development Deployment

### Docker Compose

```bash
# Clone repository
git clone https://github.com/yourusername/altis-engine.git
cd altis-engine

# Start all services
docker-compose up --build

# API available at http://localhost:8080
```

### Environment Variables

Create `.env` file:

```bash
# Database
DATABASE_URL=postgres://altis:password@postgres:5432/altis

# Redis
REDIS_URL=redis://redis:6379

# Kafka
KAFKA_BROKERS=kafka:9092

# API
SERVER_PORT=8080
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION_SECONDS=3600

# Logging
RUST_LOG=info,altis=debug
```

---

## Production Deployment (Kubernetes)

### Architecture

```
┌─────────────────────────────────────────────┐
│              Load Balancer                  │
│         (AWS ALB / GCP LB)                  │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│         Kubernetes Cluster                  │
│  ┌────────────────────────────────────┐     │
│  │  altis-api (Deployment)            │     │
│  │  Replicas: 3-10 (HPA)              │     │
│  └────────────────────────────────────┘     │
│                   │                          │
│  ┌────────────────▼──────────────────────┐  │
│  │  Managed Services                     │  │
│  │  - PostgreSQL (RDS/Cloud SQL)        │  │
│  │  - Redis (ElastiCache/Memorystore)   │  │
│  │  - Kafka (MSK/Confluent Cloud)       │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 1. Build Container Image

```bash
# Build production image
docker build -t altis-engine:latest .

# Tag for registry
docker tag altis-engine:latest gcr.io/your-project/altis-engine:v1.0.0

# Push to registry
docker push gcr.io/your-project/altis-engine:v1.0.0
```

### 2. Kubernetes Deployment

**deployment.yaml**:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: altis-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: altis-api
  template:
    metadata:
      labels:
        app: altis-api
    spec:
      containers:
      - name: api
        image: gcr.io/your-project/altis-engine:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: altis-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: altis-secrets
              key: redis-url
        - name: KAFKA_BROKERS
          value: "kafka-broker:9092"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

### 3. Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: altis-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: altis-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### 4. Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: altis-api
spec:
  selector:
    app: altis-api
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

---

## Database Setup

### PostgreSQL (AWS RDS)

```bash
# Create RDS instance
aws rds create-db-instance \
  --db-instance-identifier altis-prod \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --engine-version 15.4 \
  --master-username altis \
  --master-user-password <secure-password> \
  --allocated-storage 100 \
  --backup-retention-period 7 \
  --multi-az
```

### Run Migrations

```bash
# From local machine
export DATABASE_URL=postgres://altis:password@altis-prod.xxx.rds.amazonaws.com:5432/altis

# Run migrations
docker run --rm \
  -e DATABASE_URL=$DATABASE_URL \
  gcr.io/your-project/altis-engine:v1.0.0 \
  sqlx migrate run
```

---

## Redis Setup

### AWS ElastiCache

```bash
aws elasticache create-cache-cluster \
  --cache-cluster-id altis-redis \
  --engine redis \
  --cache-node-type cache.t3.medium \
  --num-cache-nodes 1 \
  --engine-version 7.0
```

---

## Monitoring

### Metrics

Expose Prometheus metrics:

```rust
// Add to altis-api
use prometheus::{Encoder, TextEncoder, Registry};

let registry = Registry::new();
// Register metrics...

// Metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}
```

### Key Metrics to Track

- `altis_offer_generation_duration_seconds`
- `altis_order_creation_duration_seconds`
- `altis_pricing_calculation_duration_seconds`
- `altis_offer_acceptance_rate`
- `altis_order_modification_count`

---

## Logging

### Structured Logging

```rust
use tracing::{info, error};

info!(
    order_id = %order.id,
    customer_id = %order.customer_id,
    total_nuc = order.total_nuc,
    "Order created"
);
```

### Log Aggregation

Use Fluentd/Fluent Bit to ship logs to:
- **AWS CloudWatch**
- **GCP Cloud Logging**
- **Elasticsearch**

---

## Security

### Secrets Management

Use Kubernetes Secrets or managed services:

```bash
# Create secret
kubectl create secret generic altis-secrets \
  --from-literal=database-url='postgres://...' \
  --from-literal=redis-url='redis://...' \
  --from-literal=jwt-secret='...'
```

### TLS/SSL

Enable HTTPS with cert-manager:

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: altis-tls
spec:
  secretName: altis-tls-secret
  issuerRef:
    name: letsencrypt-prod
  dnsNames:
  - api.altis.example.com
```

---

## Backup & Recovery

### Database Backups

- **Automated**: RDS automated backups (7-day retention)
- **Manual**: Daily snapshots to S3

### Disaster Recovery

- **RTO**: 1 hour (restore from snapshot)
- **RPO**: 5 minutes (point-in-time recovery)

---

## Performance Tuning

### Database Connection Pool

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
```

### Redis Connection Pool

```rust
let client = redis::Client::open(redis_url)?;
let pool = Pool::builder()
    .max_size(50)
    .build(client);
```

---

## Rollout Strategy

### Blue-Green Deployment

1. Deploy new version (green)
2. Run smoke tests
3. Switch traffic to green
4. Monitor for errors
5. Keep blue as rollback option

### Canary Deployment

1. Deploy to 10% of pods
2. Monitor metrics for 30 minutes
3. If stable, deploy to 50%
4. If stable, deploy to 100%

---

## Cost Optimization

### Estimated Monthly Costs (AWS)

| Service | Configuration | Cost |
|---------|---------------|------|
| EKS Cluster | 3 nodes (t3.medium) | $100 |
| RDS PostgreSQL | db.t3.medium | $80 |
| ElastiCache Redis | cache.t3.medium | $50 |
| MSK Kafka | kafka.t3.small (3 brokers) | $150 |
| **Total** | | **~$380/month** |

### Scaling Costs

- **10x traffic**: Add 7 API pods (~$200/month)
- **100x traffic**: Upgrade database + Redis (~$500/month)

---

## Checklist

- [ ] Build and push Docker image
- [ ] Set up managed PostgreSQL
- [ ] Set up managed Redis
- [ ] Set up managed Kafka
- [ ] Create Kubernetes secrets
- [ ] Deploy application
- [ ] Run database migrations
- [ ] Configure HPA
- [ ] Set up monitoring
- [ ] Configure logging
- [ ] Enable TLS/SSL
- [ ] Test health endpoints
- [ ] Run load tests
- [ ] Document runbooks
