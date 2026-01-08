# Kubernetes Deployment Guide

## Prerequisites

- Kubernetes cluster (1.25+)
- kubectl configured
- nginx-ingress-controller installed
- cert-manager installed (for automatic TLS)
- Persistent storage provisioner

## Quick Start

### 1. Create Namespace

```bash
kubectl create namespace ouroboros-prod
```

### 2. Setup Secrets

```bash
# Copy the secrets template
cp secrets.example.yaml secrets.yaml

# Edit with your real values
nano secrets.yaml

# Apply secrets
kubectl apply -f secrets.yaml
```

### 3. Deploy PostgreSQL

```bash
kubectl apply -f postgres.yaml
```

Wait for PostgreSQL to be ready:

```bash
kubectl wait --for=condition=ready pod -l app=postgres -n ouroboros-prod --timeout=300s
```

### 4. Run Database Migrations

```bash
# Port forward to postgres
kubectl port-forward svc/postgres 5432:5432 -n ouroboros-prod &

# Run migrations from local machine
cd ../ouro_dag
export DATABASE_URL="postgres://ouro:YOUR_PASSWORD@localhost:5432/ouro_db"
for f in migrations/*.sql; do
  psql $DATABASE_URL -f "$f"
done
```

### 5. Deploy Application

```bash
kubectl apply -f deployment.yaml
```

Wait for deployment to be ready:

```bash
kubectl rollout status deployment/ouro-node -n ouroboros-prod
```

### 6. Setup Ingress

```bash
kubectl apply -f ingress.yaml
```

### 7. Verify Deployment

```bash
# Check pods
kubectl get pods -n ouroboros-prod

# Check services
kubectl get svc -n ouroboros-prod

# Check ingress
kubectl get ingress -n ouroboros-prod

# Test health endpoint
curl https://ouroboros.example.com/health
```

## Scaling

```bash
# Scale to 5 replicas
kubectl scale deployment/ouro-node --replicas=5 -n ouroboros-prod

# Auto-scale based on CPU
kubectl autoscale deployment/ouro-node \
  --cpu-percent=70 \
  --min=3 \
  --max=10 \
  -n ouroboros-prod
```

## Monitoring

```bash
# View logs
kubectl logs -f deployment/ouro-node -n ouroboros-prod

# View specific pod logs
kubectl logs -f ouro-node-xxxxx -n ouroboros-prod

# View logs from all pods
kubectl logs -l app=ouro-node -n ouroboros-prod --tail=100
```

## Backup & Restore

### Backup PostgreSQL

```bash
kubectl exec -n ouroboros-prod \
  deployment/postgres -- \
  pg_dump -U ouro ouro_db > backup-$(date +%Y%m%d-%H%M%S).sql
```

### Restore PostgreSQL

```bash
kubectl exec -i -n ouroboros-prod \
  deployment/postgres -- \
  psql -U ouro ouro_db < backup-20250123-120000.sql
```

## Troubleshooting

### Pod not starting

```bash
# Describe pod to see events
kubectl describe pod ouro-node-xxxxx -n ouroboros-prod

# Check logs
kubectl logs ouro-node-xxxxx -n ouroboros-prod

# Check if secrets are correctly mounted
kubectl exec -it ouro-node-xxxxx -n ouroboros-prod -- env | grep -E "DATABASE|API"
```

### Database connection issues

```bash
# Test postgres connectivity
kubectl run -it --rm debug --image=postgres:16 -n ouroboros-prod -- \
  psql postgres://ouro:PASSWORD@postgres:5432/ouro_db
```

### Ingress not working

```bash
# Check ingress controller logs
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller

# Check certificate status (if using cert-manager)
kubectl get certificate -n ouroboros-prod
kubectl describe certificate ouro-tls-cert -n ouroboros-prod
```

## Production Checklist

- [ ] Strong passwords in secrets.yaml
- [ ] API keys rotated from defaults
- [ ] TLS certificates configured (cert-manager or manual)
- [ ] Resource limits tuned for workload
- [ ] Monitoring/alerting configured (Prometheus/Grafana)
- [ ] Backup strategy implemented
- [ ] Disaster recovery plan documented
- [ ] Rate limiting configured in ingress
- [ ] Network policies applied
- [ ] RBAC configured
- [ ] Audit logging enabled

## Updating the Application

### Rolling Update

```bash
# Update image tag
kubectl set image deployment/ouro-node \
  ouro-node=ghcr.io/your-org/ouroboros:v1.2.0 \
  -n ouroboros-prod

# Watch rollout
kubectl rollout status deployment/ouro-node -n ouroboros-prod
```

### Rollback

```bash
# Rollback to previous version
kubectl rollout undo deployment/ouro-node -n ouroboros-prod

# Rollback to specific revision
kubectl rollout undo deployment/ouro-node --to-revision=2 -n ouroboros-prod
```

## Clean Up

```bash
# Delete all resources
kubectl delete -f ingress.yaml
kubectl delete -f deployment.yaml
kubectl delete -f postgres.yaml
kubectl delete -f secrets.yaml

# Delete namespace (warning: deletes everything!)
kubectl delete namespace ouroboros-prod
```
