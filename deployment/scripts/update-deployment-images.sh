#!/bin/bash

# Update Kubernetes Deployment Images
# Updates the deployment manifests to use newly built images

set -e

NAMESPACE="${NAMESPACE:-pixelcore}"
BACKEND_IMAGE="${BACKEND_IMAGE:-pixelcore/backend:latest}"
FRONTEND_IMAGE="${FRONTEND_IMAGE:-pixelcore/frontend:latest}"

echo "=========================================="
echo "Updating Kubernetes Deployments"
echo "=========================================="
echo "Namespace: $NAMESPACE"
echo "Backend Image: $BACKEND_IMAGE"
echo "Frontend Image: $FRONTEND_IMAGE"
echo ""

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "Error: kubectl not found"
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info &> /dev/null; then
    echo "Error: Cannot connect to Kubernetes cluster"
    exit 1
fi

echo "1. Updating backend deployment..."
kubectl set image deployment/backend backend=$BACKEND_IMAGE -n $NAMESPACE
echo "✓ Backend deployment updated"
echo ""

echo "2. Updating frontend deployment..."
kubectl set image deployment/frontend frontend=$FRONTEND_IMAGE -n $NAMESPACE
echo "✓ Frontend deployment updated"
echo ""

echo "3. Waiting for rollout to complete..."
kubectl rollout status deployment/backend -n $NAMESPACE --timeout=5m
kubectl rollout status deployment/frontend -n $NAMESPACE --timeout=5m
echo "✓ Rollout complete"
echo ""

echo "4. Checking pod status..."
kubectl get pods -n $NAMESPACE
echo ""

echo "=========================================="
echo "Deployment Updated Successfully!"
echo "=========================================="
