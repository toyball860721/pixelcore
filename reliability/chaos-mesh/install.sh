#!/bin/bash
# Chaos Mesh installation script

set -euo pipefail

echo "Installing Chaos Mesh..."

# Add Chaos Mesh Helm repository
helm repo add chaos-mesh https://charts.chaos-mesh.org
helm repo update

# Create namespace
kubectl create namespace chaos-mesh --dry-run=client -o yaml | kubectl apply -f -

# Install Chaos Mesh
helm install chaos-mesh chaos-mesh/chaos-mesh \
  --namespace=chaos-mesh \
  --set chaosDaemon.runtime=containerd \
  --set chaosDaemon.socketPath=/run/containerd/containerd.sock \
  --set dashboard.create=true \
  --set dashboard.securityMode=false \
  --version 2.6.3

echo "Waiting for Chaos Mesh to be ready..."
kubectl wait --for=condition=Ready pods --all -n chaos-mesh --timeout=300s

echo "Chaos Mesh installed successfully!"
echo ""
echo "Access the dashboard:"
echo "  kubectl port-forward -n chaos-mesh svc/chaos-dashboard 2333:2333"
echo "  Then open: http://localhost:2333"
