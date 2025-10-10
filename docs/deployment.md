# Deployment Guide

This guide covers deploying langgraph-rs in various environments.

## Docker

### Build Docker Image

```bash
docker build -t langgraph-rs:latest .
```

### Run Container

```bash
docker run -it --rm \
  -v $(pwd)/examples:/app/examples \
  -v $(pwd)/output:/app/output \
  langgraph-rs:latest convert examples/simple_workflow.py
```

### Docker Compose

For a complete setup with monitoring:

```bash
docker-compose up -d
```

Access services:
- LangGraph-RS: http://localhost:8080
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)

## Kubernetes

### Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- Container registry access

### Deploy to Kubernetes

1. **Build and push image:**
   ```bash
   docker build -t your-registry/langgraph-rs:latest .
   docker push your-registry/langgraph-rs:latest
   ```

2. **Update image in deployment:**
   ```bash
   # Edit k8s/deployment.yaml and update image field
   vim k8s/deployment.yaml
   ```

3. **Apply manifests:**
   ```bash
   kubectl apply -f k8s/configmap.yaml
   kubectl apply -f k8s/deployment.yaml
   kubectl apply -f k8s/service.yaml
   ```

4. **Verify deployment:**
   ```bash
   kubectl get pods -l app=langgraph-rs
   kubectl logs -l app=langgraph-rs
   ```

### Scaling

```bash
kubectl scale deployment langgraph-rs --replicas=5
```

### Health Checks

The application exposes health endpoints:
- `/health`: Liveness probe
- `/ready`: Readiness probe

## Performance Tuning

### Environment Variables

```bash
# Logging
export RUST_LOG=info

# Performance
export TOKIO_WORKER_THREADS=4

# Caching
export CACHE_SIZE=10000
```

### Resource Limits

**Recommended for production:**

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "200m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### Connection Pooling

Configure in `config.toml`:

```toml
[performance]
connection_pool_size = 20
request_timeout_secs = 30
max_concurrent_requests = 100
```

## Monitoring

### Prometheus Metrics

Metrics are exposed at `/metrics`:

- `langgraph_conversions_total`: Total conversions
- `langgraph_conversion_duration_seconds`: Conversion time
- `langgraph_errors_total`: Error count
- `langgraph_cache_hits_total`: Cache hits
- `langgraph_cache_misses_total`: Cache misses

### Grafana Dashboards

Import the provided dashboard:

```bash
# Located in monitoring/grafana/dashboards/langgraph-rs.json
```

### Logging

Structured JSON logging for production:

```bash
export RUST_LOG=info
export LOG_FORMAT=json
```

## Cloud Deployments

### AWS ECS

```bash
# Create task definition
aws ecs register-task-definition --cli-input-json file://aws/task-definition.json

# Create service
aws ecs create-service --cli-input-json file://aws/service.json
```

### Google Cloud Run

```bash
gcloud run deploy langgraph-rs \
  --image gcr.io/PROJECT_ID/langgraph-rs:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

### Azure Container Instances

```bash
az container create \
  --resource-group myResourceGroup \
  --name langgraph-rs \
  --image myregistry.azurecr.io/langgraph-rs:latest \
  --cpu 2 \
  --memory 4
```

## Security

### Best Practices

1. **Use non-root user:** Already configured in Dockerfile
2. **Scan images:** Use trivy or similar tools
3. **Secret management:** Use Kubernetes secrets or cloud secret managers
4. **Network policies:** Restrict pod-to-pod communication
5. **RBAC:** Implement least-privilege access

### Secrets

```bash
# Create secret for API keys
kubectl create secret generic langgraph-secrets \
  --from-literal=openai-api-key=YOUR_KEY
```

Mount in deployment:

```yaml
env:
- name: OPENAI_API_KEY
  valueFrom:
    secretKeyRef:
      name: langgraph-secrets
      key: openai-api-key
```

## Troubleshooting

### Common Issues

**Out of Memory:**
```bash
# Increase memory limit in deployment
resources:
  limits:
    memory: "2Gi"
```

**Slow Performance:**
```bash
# Check metrics
kubectl logs -l app=langgraph-rs | grep duration

# Scale up
kubectl scale deployment langgraph-rs --replicas=5
```

**Connection Issues:**
```bash
# Check service
kubectl get svc langgraph-rs

# Test connectivity
kubectl run -it --rm debug --image=busybox --restart=Never -- wget -O- http://langgraph-rs/health
```

## Backup and Recovery

### State Backup

```bash
# If using persistent storage
kubectl exec -it langgraph-rs-xxx -- tar czf /tmp/backup.tar.gz /app/data
kubectl cp langgraph-rs-xxx:/tmp/backup.tar.gz ./backup.tar.gz
```

### Disaster Recovery

1. Keep images in multiple registries
2. Use infrastructure as code (Terraform/Pulumi)
3. Regular backup of configuration
4. Document recovery procedures

## CI/CD Integration

### GitHub Actions

Already configured in `.github/workflows/`:
- Build and test on PR
- Build and push Docker image on main
- Deploy to staging/production

### GitLab CI

```yaml
# .gitlab-ci.yml example
build:
  stage: build
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA

deploy:
  stage: deploy
  script:
    - kubectl set image deployment/langgraph-rs langgraph-rs=$CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
```

## Support

For deployment issues:
- GitHub Issues: https://github.com/genai-rs/langgraph-rs/issues
- Documentation: https://docs.langgraph-rs.io
- Email: tim.van.wassenhove@gmail.com
