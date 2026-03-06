#!/bin/bash

# PixelCore Istio Installation Script
# This script installs and configures Istio for the PixelCore service mesh

set -e

echo "🚀 Installing Istio for PixelCore..."

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed. Please install kubectl first."
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster. Please check your kubeconfig."
    exit 1
fi

# Download Istio
ISTIO_VERSION="1.20.0"
echo "📦 Downloading Istio ${ISTIO_VERSION}..."

if [ ! -d "istio-${ISTIO_VERSION}" ]; then
    curl -L https://istio.io/downloadIstio | ISTIO_VERSION=${ISTIO_VERSION} sh -
fi

cd istio-${ISTIO_VERSION}
export PATH=$PWD/bin:$PATH

# Install Istio with production profile
echo "⚙️  Installing Istio with production profile..."
istioctl install --set profile=production -y

# Enable automatic sidecar injection for pixelcore namespace
echo "💉 Enabling automatic sidecar injection..."
kubectl create namespace pixelcore --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace pixelcore istio-injection=enabled --overwrite

# Install Istio addons (Prometheus, Grafana, Jaeger, Kiali)
echo "📊 Installing observability addons..."
kubectl apply -f samples/addons/prometheus.yaml
kubectl apply -f samples/addons/grafana.yaml
kubectl apply -f samples/addons/jaeger.yaml
kubectl apply -f samples/addons/kiali.yaml

# Wait for Istio components to be ready
echo "⏳ Waiting for Istio components to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/istiod -n istio-system
kubectl wait --for=condition=available --timeout=300s deployment/istio-ingressgateway -n istio-system

echo "✅ Istio installation completed successfully!"
echo ""
echo "📝 Next steps:"
echo "  1. Apply gateway configuration: kubectl apply -f gateway/"
echo "  2. Apply virtual services: kubectl apply -f virtual-services/"
echo "  3. Apply destination rules: kubectl apply -f destination-rules/"
echo ""
echo "🔍 Access dashboards:"
echo "  - Kiali: kubectl port-forward svc/kiali -n istio-system 20001:20001"
echo "  - Grafana: kubectl port-forward svc/grafana -n istio-system 3000:3000"
echo "  - Jaeger: kubectl port-forward svc/tracing -n istio-system 16686:16686"
echo "  - Prometheus: kubectl port-forward svc/prometheus -n istio-system 9090:9090"
