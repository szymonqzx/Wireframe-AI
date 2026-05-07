# Deployment Guide

## Docker Compose (Single Node)

```bash
cd deploy/docker
docker-compose -f docker-compose.prod.yml up -d
```

This starts:
- NATS with JetStream persistence
- Core modules (context, orchestrator, sandbox)
- Platform modules (provider-router-core, event-sourcing-core, observability-core, tenant-core, webhooks-core, integrations-core)
- Monitoring (Prometheus + Grafana)

## Kubernetes (Production)

### Prerequisites

- Kubernetes 1.28+
- kubectl configured
- Docker registry access

### Build Images

```bash
# Build all module images
for module in context orchestrator sandbox provider-router-core event-sourcing-core observability-core tenant-core webhooks-core integrations-core; do
  docker build \
    -f deploy/docker/Dockerfile.module \
    --build-arg MODULE=modules/$module \
    --build-arg BINARY_NAME=wireframe-ai-$module \
    -t wireframe-ai-$module:latest .
done
```

### Deploy

```bash
kubectl apply -f deploy/k8s/manifests/namespace.yaml
kubectl apply -f deploy/k8s/manifests/nats.yaml
kubectl apply -f deploy/k8s/manifests/context.yaml
# Apply additional module manifests as needed
```

### Verify

```bash
kubectl get pods -n wireframe-ai
kubectl logs -n wireframe-ai deployment/wireframe-context
```

## Helm Chart (Optional)

A Helm chart is recommended for complex deployments:

```bash
helm install wireframe-ai ./deploy/helm \
  --namespace wireframe-ai \
  --set nats.replicas=3 \
  --set modules.orchestrator.replicas=2
```

## Disaster Recovery

### Backup

1. NATS JetStream data (persistent volume snapshots)
2. Context module SQLite database
3. Event sourcing event log exports

### Restore

1. Restore NATS volumes
2. Restart modules in order: NATS -> Context -> Orchestrator -> Others
3. Verify with `wireframe debug --topic sys.module.online`

## Scaling

- **Horizontal**: Increase module replica counts (stateless modules)
- **Vertical**: Increase CPU/memory limits in manifests
- **NATS**: Scale NATS cluster for higher throughput

## Monitoring

Access dashboards:
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)

Key metrics:
- `wireframe_messages_per_second`
- `wireframe_module_latency_ms`
- `wireframe_provider_cost_cents`
- `wireframe_tenant_quota_usage`
