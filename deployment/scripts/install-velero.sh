#!/bin/bash
# Velero 备份系统安装脚本

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 配置
VELERO_VERSION="${VELERO_VERSION:-v1.12.0}"
CLOUD_PROVIDER="${CLOUD_PROVIDER:-aws}"
BUCKET_NAME="${BUCKET_NAME:-pixelcore-backups}"
REGION="${REGION:-us-west-2}"

log_info "安装Velero备份系统..."
log_info "版本: $VELERO_VERSION"
log_info "云提供商: $CLOUD_PROVIDER"
log_info "存储桶: $BUCKET_NAME"
log_info "区域: $REGION"

# 检查velero CLI
if ! command -v velero &> /dev/null; then
    log_error "velero CLI未安装"
    log_info "请访问: https://velero.io/docs/main/basic-install/"
    exit 1
fi

# 根据云提供商安装
case $CLOUD_PROVIDER in
    aws)
        log_info "安装AWS插件..."
        velero install \
            --provider aws \
            --plugins velero/velero-plugin-for-aws:v1.8.0 \
            --bucket "$BUCKET_NAME" \
            --backup-location-config region="$REGION" \
            --snapshot-location-config region="$REGION" \
            --secret-file ./credentials-velero \
            --use-volume-snapshots=true \
            --use-node-agent
        ;;
    gcp)
        log_info "安装GCP插件..."
        velero install \
            --provider gcp \
            --plugins velero/velero-plugin-for-gcp:v1.8.0 \
            --bucket "$BUCKET_NAME" \
            --secret-file ./credentials-velero
        ;;
    azure)
        log_info "安装Azure插件..."
        velero install \
            --provider azure \
            --plugins velero/velero-plugin-for-microsoft-azure:v1.8.0 \
            --bucket "$BUCKET_NAME" \
            --secret-file ./credentials-velero \
            --backup-location-config resourceGroup="$AZURE_RESOURCE_GROUP",storageAccount="$AZURE_STORAGE_ACCOUNT" \
            --snapshot-location-config apiTimeout="$AZURE_API_TIMEOUT"
        ;;
    *)
        log_error "不支持的云提供商: $CLOUD_PROVIDER"
        exit 1
        ;;
esac

# 等待Velero就绪
log_info "等待Velero就绪..."
kubectl wait --for=condition=Ready pods --all -n velero --timeout=300s

# 验证安装
log_info "验证Velero安装..."
velero version

log_info "Velero安装完成 ✓"
log_info ""
log_info "下一步："
log_info "  1. 部署备份计划: kubectl apply -f ../../reliability/velero-schedules.yaml"
log_info "  2. 验证备份: velero backup get"
log_info "  3. 测试恢复: velero restore create --from-backup <backup-name>"
