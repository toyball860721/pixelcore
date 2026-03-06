#!/bin/bash

# PixelCore Velero Installation Script
# Installs Velero for backup and disaster recovery

set -e

echo "🔄 Installing Velero for PixelCore..."

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed"
    exit 1
fi

# Check if helm is installed
if ! command -v helm &> /dev/null; then
    echo "❌ helm is not installed"
    echo "Install: brew install helm"
    exit 1
fi

# Configuration
VELERO_VERSION="v1.12.0"
BACKUP_LOCATION="s3"
BUCKET_NAME="${VELERO_BUCKET:-pixelcore-backups}"
REGION="${AWS_REGION:-us-east-1}"

echo "📦 Adding Velero Helm repository..."
helm repo add vmware-tanzu https://vmware-tanzu.github.io/helm-charts
helm repo update

echo "⚙️  Installing Velero..."
helm install velero vmware-tanzu/velero \
  --namespace velero \
  --create-namespace \
  --set configuration.provider=aws \
  --set configuration.backupStorageLocation.bucket=$BUCKET_NAME \
  --set configuration.backupStorageLocation.config.region=$REGION \
  --set configuration.volumeSnapshotLocation.config.region=$REGION \
  --set initContainers[0].name=velero-plugin-for-aws \
  --set initContainers[0].image=velero/velero-plugin-for-aws:v1.8.0 \
  --set initContainers[0].volumeMounts[0].mountPath=/target \
  --set initContainers[0].volumeMounts[0].name=plugins \
  --set credentials.useSecret=true \
  --set credentials.existingSecret=cloud-credentials

# Wait for Velero to be ready
echo "⏳ Waiting for Velero to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/velero -n velero

echo "✅ Velero installation completed!"
echo ""
echo "📝 Next steps:"
echo "  1. Create AWS credentials secret:"
echo "     kubectl create secret generic cloud-credentials \\"
echo "       --namespace velero \\"
echo "       --from-file=cloud=./credentials-velero"
echo ""
echo "  2. Create a backup:"
echo "     velero backup create pixelcore-backup --include-namespaces pixelcore"
echo ""
echo "  3. Schedule automatic backups:"
echo "     velero schedule create pixelcore-daily --schedule='0 2 * * *' --include-namespaces pixelcore"
echo ""
echo "  4. List backups:"
echo "     velero backup get"
