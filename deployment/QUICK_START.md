# Quick Start Guide - Local Test Environment

## Current Status
✅ Local Kind cluster is running with all components deployed and healthy

## Quick Commands

### Check Deployment Status
```bash
# View all pods
kubectl get pods -n pixelcore

# Run health check
./deployment/scripts/health-check.sh

# View services
kubectl get services -n pixelcore
```

### Access Services
```bash
# Access frontend (port 8080)
kubectl port-forward -n pixelcore svc/frontend-service 8080:80

# Access backend (port 8081)
kubectl port-forward -n pixelcore svc/backend-service 8081:8080

# Access PostgreSQL (port 5432)
kubectl port-forward -n pixelcore svc/postgres-service 5432:5432

# Access Redis (port 6379)
kubectl port-forward -n pixelcore svc/redis-service 6379:6379
```

### Test Operations
```bash
# Scale backend
kubectl scale deployment backend -n pixelcore --replicas=5

# Test pod recovery
kubectl delete pod <pod-name> -n pixelcore

# View logs
kubectl logs -n pixelcore <pod-name> -f

# Execute commands in pod
kubectl exec -it -n pixelcore <pod-name> -- sh
```

### Cleanup
```bash
# Delete cluster
kind delete cluster --name pixelcore-test

# Verify cleanup
kind get clusters
```

## What's Next?

### Option 1: Continue Testing
- Test scaling and failover
- Experiment with configurations
- Develop and debug applications

### Option 2: Build Real Images
- Build backend Docker image
- Build frontend Docker image
- Deploy with real application code

### Option 3: Production Deployment
- Configure production environment
- Set up monitoring and backups
- Run `./deployment/scripts/deploy-production.sh`

## Files Created
- `deployment/scripts/setup-local-test.sh` - Cluster setup
- `deployment/scripts/deploy-test.sh` - Test deployment
- `deployment/scripts/deploy-demo.sh` - Demo deployment
- `deployment/scripts/health-check.sh` - Health validation
- `deployment/TEST_DEPLOYMENT_SUMMARY.md` - Detailed summary
- `deployment/QUICK_START.md` - This guide
