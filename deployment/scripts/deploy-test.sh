#!/bin/bash
# 测试环境部署脚本（简化版）
# 适用于本地Kind集群

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

NAMESPACE="pixelcore"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "PixelCore 测试环境部署"
echo "========================================="
echo ""

# 检查集群
check_cluster() {
    log_step "检查Kubernetes集群..."

    if ! kubectl cluster-info &> /dev/null; then
        log_error "无法连接到Kubernetes集群"
        log_info "请先运行: ./setup-local-test.sh"
        exit 1
    fi

    log_info "集群连接正常 ✓"
}

# 部署PostgreSQL（单副本测试版）
deploy_postgres() {
    log_step "部署PostgreSQL（测试版）..."

    kubectl apply -f "$PROJECT_ROOT/k8s/base/postgres.yaml"

    log_info "等待PostgreSQL就绪..."
    kubectl wait --for=condition=Ready pods -l app=postgres -n "$NAMESPACE" --timeout=300s || true

    log_info "PostgreSQL部署完成 ✓"
}

# 部署Redis（单副本测试版）
deploy_redis() {
    log_step "部署Redis（测试版）..."

    kubectl apply -f "$PROJECT_ROOT/k8s/base/redis.yaml"

    log_info "等待Redis就绪..."
    kubectl wait --for=condition=Ready pods -l app=redis -n "$NAMESPACE" --timeout=300s || true

    log_info "Redis部署完成 ✓"
}

# 部署应用
deploy_application() {
    log_step "部署应用服务..."

    # ConfigMap和Secret
    kubectl apply -f "$PROJECT_ROOT/k8s/base/configmap.yaml"
    kubectl apply -f "$PROJECT_ROOT/k8s/base/secret.yaml"

    # 后端服务
    kubectl apply -f "$PROJECT_ROOT/k8s/base/backend.yaml"

    # 前端服务
    kubectl apply -f "$PROJECT_ROOT/k8s/base/frontend.yaml"

    # HPA和PDB
    kubectl apply -f "$PROJECT_ROOT/k8s/base/hpa.yaml"
    kubectl apply -f "$PROJECT_ROOT/k8s/base/pdb.yaml"

    # Ingress
    kubectl apply -f "$PROJECT_ROOT/k8s/base/ingress.yaml"

    log_info "等待应用就绪..."
    sleep 10
    kubectl wait --for=condition=Ready pods -l app=backend -n "$NAMESPACE" --timeout=300s || true
    kubectl wait --for=condition=Ready pods -l app=frontend -n "$NAMESPACE" --timeout=300s || true

    log_info "应用部署完成 ✓"
}

# 显示部署状态
show_status() {
    log_step "部署状态"

    echo ""
    echo "Pods:"
    kubectl get pods -n "$NAMESPACE"

    echo ""
    echo "Services:"
    kubectl get svc -n "$NAMESPACE"

    echo ""
    echo "HPA:"
    kubectl get hpa -n "$NAMESPACE"

    echo ""
    echo "PDB:"
    kubectl get pdb -n "$NAMESPACE"
}

# 显示访问信息
show_access_info() {
    echo ""
    echo "========================================="
    echo "测试环境部署完成！"
    echo "========================================="
    echo ""
    echo "访问应用："
    echo "  kubectl port-forward -n $NAMESPACE svc/backend-service 8080:8080"
    echo "  kubectl port-forward -n $NAMESPACE svc/frontend-service 3000:3000"
    echo ""
    echo "查看日志："
    echo "  kubectl logs -f -l app=backend -n $NAMESPACE"
    echo "  kubectl logs -f -l app=frontend -n $NAMESPACE"
    echo ""
    echo "运行健康检查："
    echo "  cd ../validation && ./health-check.sh"
    echo ""
    echo "========================================="
}

# 主函数
main() {
    check_cluster
    deploy_postgres
    deploy_redis
    deploy_application
    show_status
    show_access_info

    log_info "测试环境部署完成！🎉"
}

main "$@"
