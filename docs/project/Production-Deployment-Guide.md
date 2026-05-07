# Wireframe-AI Production Deployment Guide

This guide provides comprehensive strategies for deploying Wireframe-AI to production environments.

## Table of Contents

- [Pre-Deployment Checklist](#pre-deployment-checklist)
- [Deployment Strategies](#deployment-strategies)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Configuration Management](#configuration-management)
- [Monitoring Setup](#monitoring-setup)
- [Security Hardening](#security-hardening)
- [Backup and Recovery](#backup-and-recovery)
- [Scaling Strategies](#scaling-strategies)
- [Troubleshooting](#troubleshooting)

## Pre-Deployment Checklist

### 1. Configuration

- [ ] Production configuration file created and validated
- [ ] Environment variables set for sensitive data
- [ ] Secrets management configured
- [ ] NATS server configured for production
- [ ] Database connection strings verified

### 2. Security

- [ ] API keys rotated to production values
- [ ] TLS/SSL enabled for all endpoints
- [ ] Firewall rules configured
- [ ] Authentication enabled
- [ ] Security policies reviewed

### 3. Monitoring

- [ ] Metrics collection enabled
- [ ] Tracing configured
- [ ] Alert rules set up
- [ ] Dashboard configured
- [ ] Log aggregation configured

### 4. Testing

- [ ] Integration tests passing
- [ ] Load tests completed
- [ ] Security audit performed
- [ ] Performance benchmarks met
- [ ] Disaster recovery tested

### 5. Documentation

- [ ] Runbook created
- [ ] On-call procedures documented
- [ ] Rollback procedures tested
- [ ] Architecture diagrams updated
- [ ] Known issues documented

## Deployment Strategies

### 1. Docker Compose

Simple deployment for single-server setups.

**Pros:**
- Easy to set up
- Good for small deployments
- Simple to maintain

**Cons:**
- Limited scalability
- Single point of failure
- Manual scaling

### 2. Docker Swarm

Container orchestration for multi-server deployments.

**Pros:**
- Built-in orchestration
- Load balancing
- Service discovery
- Easier than Kubernetes

**Cons:**
- Less mature than Kubernetes
- Fewer features
- Smaller ecosystem

### 3. Kubernetes

Production-grade orchestration for large-scale deployments.

**Pros:**
- Highly scalable
- Self-healing
- Rich ecosystem
- Advanced features

**Cons:**
- Steep learning curve
- Complex setup
- Overhead for small deployments

### 4. Cloud-Native Services

Use managed services (AWS, GCP, Azure).

**Pros:**
- Managed infrastructure
- Auto-scaling
- Built-in monitoring
- Reduced operational burden

**Cons:**
- Vendor lock-in
- Higher cost
- Less control

## Docker Deployment

### Docker Compose Setup

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  nats:
    image: nats:latest
    ports:
      - "4222:4222"
    command: "-js"
    volumes:
      - nats-data:/data

  context-core:
    build:
      context: .
      dockerfile: Dockerfile.context
    environment:
      - WIREFRAME_NATS_URL=nats://nats:4222
      - RUST_LOG=info
    depends_on:
      - nats
    restart: unless-stopped

  orchestrator-core:
    build:
      context: .
      dockerfile: Dockerfile.orchestrator
    environment:
      - WIREFRAME_NATS_URL=nats://nats:4222
      - RUST_LOG=info
    depends_on:
      - nats
    restart: unless-stopped

  sandbox-core:
    build:
      context: .
      dockerfile: Dockerfile.sandbox
    environment:
      - WIREFRAME_NATS_URL=nats://nats:4222
      - RUST_LOG=info
    depends_on:
      - nats
    restart: unless-stopped

  interface-core:
    build:
      context: .
      dockerfile: Dockerfile.interface
    environment:
      - WIREFRAME_NATS_URL=nats://nats:4222
      - RUST_LOG=info
    depends_on:
      - nats
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./configs/prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    depends_on:
      - prometheus
    restart: unless-stopped

volumes:
  nats-data:
  grafana-data:
```

### Dockerfile Example

Create `Dockerfile.context`:

```dockerfile
FROM rust:1.80 as builder

WORKDIR /app
COPY . .
RUN cargo build --release -p context-core

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/context-core /app/

ENV RUST_LOG=info
EXPOSE 8080

CMD ["./context-core"]
```

### Deployment Commands

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Scale services
docker-compose up -d --scale context-core=3

# Stop services
docker-compose down
```

## Kubernetes Deployment

### Namespace and ConfigMap

Create `k8s/namespace.yaml`:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: wireframe-ai
```

Create `k8s/configmap.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wireframe-config
  namespace: wireframe-ai
data:
  NATS_URL: "nats://nats:4222"
  RUST_LOG: "info"
```

### NATS Deployment

Create `k8s/nats-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
  namespace: wireframe-ai
spec:
  serviceName: nats
  replicas: 3
  selector:
    matchLabels:
      app: nats
  template:
    metadata:
      labels:
        app: nats
    spec:
      containers:
      - name: nats
        image: nats:latest
        ports:
        - containerPort: 4222
        command: ["nats-server", "-js"]
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: nats
  namespace: wireframe-ai
spec:
  selector:
    app: nats
  ports:
  - port: 4222
    targetPort: 4222
```

### Module Deployment

Create `k8s/context-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: context-core
  namespace: wireframe-ai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: context-core
  template:
    metadata:
      labels:
        app: context-core
    spec:
      containers:
      - name: context-core
        image: wireframe-ai/context-core:latest
        env:
        - name: WIREFRAME_NATS_URL
          valueFrom:
            configMapKeyRef:
              name: wireframe-config
              key: NATS_URL
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: wireframe-config
              key: RUST_LOG
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: context-core
  namespace: wireframe-ai
spec:
  selector:
    app: context-core
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

### Horizontal Pod Autoscaler

Create `k8s/hpa.yaml`:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: context-core-hpa
  namespace: wireframe-ai
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: context-core
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Deployment Commands

```bash
# Apply all configurations
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n wireframe-ai

# View logs
kubectl logs -f deployment/context-core -n wireframe-ai

# Scale deployment
kubectl scale deployment context-core --replicas=5 -n wireframe-ai
```

## Configuration Management

### Environment Variables

Use environment variables for configuration:

```yaml
env:
  - name: WIREFRAME_NATS_URL
    valueFrom:
      secretKeyRef:
        name: wireframe-secrets
        key: nats-url
  - name: OPENAI_API_KEY
    valueFrom:
      secretKeyRef:
        name: wireframe-secrets
        key: openai-api-key
```

### Secrets Management

Create Kubernetes secret:

```bash
kubectl create secret generic wireframe-secrets \
  --from-literal=nats-url='nats://user:password@nats.example.com:4222' \
  --from-literal=openai-api-key='sk-...' \
  -n wireframe-ai
```

### ConfigMaps vs Secrets

- **ConfigMaps**: Non-sensitive configuration
- **Secrets**: Sensitive data (API keys, passwords)

## Monitoring Setup

### Prometheus Configuration

Create `configs/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'wireframe-context'
    static_configs:
      - targets: ['context-core:8080']
  - job_name: 'wireframe-orchestrator'
    static_configs:
      - targets: ['orchestrator-core:8080']
  - job_name: 'wireframe-sandbox'
    static_configs:
      - targets: ['sandbox-core:8080']
```

### Grafana Dashboard

Import the provided dashboard configuration to visualize metrics.

### Alert Rules

Create alert rules in Prometheus:

```yaml
groups:
  - name: wireframe-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(wireframe_errors_total[5m]) > 0.1
        for: 5m
        annotations:
          summary: "High error rate detected"
      - alert: HighLatency
        expr: histogram_quantile(0.95, task_duration_seconds) > 60
        for: 5m
        annotations:
          summary: "High task latency detected"
```

## Security Hardening

### 1. Network Policies

Create Kubernetes network policies:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: wireframe-network-policy
  namespace: wireframe-ai
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: wireframe-ai
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: wireframe-ai
```

### 2. Pod Security Policies

```yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: wireframe-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'secret'
    - 'emptyDir'
  hostNetwork: false
  hostIPC: false
  hostPID: false
  runAsUser:
    rule: 'MustRunAsNonRoot'
```

### 3. Resource Limits

Always set resource limits:

```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### 4. Image Security

- Use official images when possible
- Scan images for vulnerabilities
- Use specific image tags (not `latest`)
- Sign images with cosign

## Backup and Recovery

### 1. Database Backups

```bash
# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump -U wireframe wireframe_db > /backups/wireframe_$DATE.sql
```

### 2. Configuration Backups

```bash
# Backup all configs
kubectl get configmaps -n wireframe-ai -o yaml > /backups/configmaps.yaml
kubectl get secrets -n wireframe-ai -o yaml > /backups/secrets.yaml
```

### 3. Disaster Recovery

Document recovery procedures:
1. Restore from backup
2. Verify data integrity
3. Restart services
4. Monitor for issues

## Scaling Strategies

### 1. Horizontal Scaling

Add more replicas:

```bash
kubectl scale deployment context-core --replicas=10 -n wireframe-ai
```

### 2. Vertical Scaling

Increase resource limits:

```yaml
resources:
  limits:
    memory: "2Gi"
    cpu: "2000m"
```

### 3. Auto-scaling

Configure HPA for automatic scaling:

```bash
kubectl autoscale deployment context-core \
  --min=3 --max=10 \
  --cpu-percent=70 \
  -n wireframe-ai
```

### 4. Database Scaling

- Use read replicas for read-heavy workloads
- Implement connection pooling
- Use database sharding for very large deployments

## Troubleshooting

### 1. Check Pod Status

```bash
kubectl get pods -n wireframe-ai
kubectl describe pod <pod-name> -n wireframe-ai
```

### 2. View Logs

```bash
kubectl logs -f deployment/context-core -n wireframe-ai
kubectl logs <pod-name> -n wireframe-ai --previous
```

### 3. Debug Pods

```bash
kubectl exec -it <pod-name> -n wireframe-ai -- /bin/bash
```

### 4. Check Events

```bash
kubectl get events -n wireframe-ai --sort-by='.lastTimestamp'
```

### 5. Resource Issues

```bash
kubectl top pods -n wireframe-ai
kubectl top nodes
```

## Rollback Procedures

### 1. Kubernetes Rollback

```bash
kubectl rollout undo deployment/context-core -n wireframe-ai
```

### 2. Database Rollback

```bash
# Restore from backup
psql -U wireframe -d wireframe_db < /backups/wireframe_YYYYMMDD.sql
```

### 3. Configuration Rollback

```bash
kubectl apply -f k8s/configmap-previous.yaml
kubectl rollout restart deployment/context-core -n wireframe-ai
```

## Next Steps

- See [Performance Optimization Guide](./Performance-Optimization-Guide.md) for optimization techniques
- See [Monitoring README](../monitoring/README.md) for monitoring setup
- See [Load Testing Script](../scripts/load-test.sh) for testing
