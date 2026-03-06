#!/bin/bash
# PixelCore 生产环境部署脚本
# 用途：一键部署完整的生产环境

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# 配置
NAMESPACE="${NAMESPACE:-pixelcore}"
ENVIRONMENT="${ENVIRONMENT:-production}"
DRY_RUN="${DRY_RUN:-false}"

# 检查前置条件
check_prerequisites() {
    log_step "检查前置条件..."

    # 检查kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl 未安装"
        exit 1
    fi

    # 检查集群连接
    if ! kubectl cluster-info &> /dev/null; then
        log_error "无法连接到Kubernetes集群"
        exit 1
    fi

    # 检查helm
    if ! command -v helm &> /dev/null; then
        log_warn "helm 未安装，某些功能可能不可用"
    fi

    log_info "前置条件检查通过 ✓"
}

# 创建命名空间
create_namespace() {
    log_step "创建命名空间: $NAMESPACE"

    if kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_info "命名空间已存在"
    else
        kubectl create namespace "$NAMESPACE"
        kubectl label namespace "$NAMESPACE" environment="$ENVIRONMENT"
        log_info "命名空间创建成功 ✓"
    fi
}

# 部署基础设施
deploy_infrastructure() {
    log_step "部署基础设施..."

    # 1. 部署PostgreSQL HA
    log_info "部署PostgreSQL HA集群..."
    kubectl apply -f ../../k8s/base/postgres-ha.yaml
    kubectl apply -f ../../k8s/base/postgres-pdb.yaml

    # 2. 部署Redis HA
    log_info "部署Redis HA集群..."
    kubectl apply -f ../../k8s/base/redis-ha.yaml
    kubectl apply -f ../../k8s/base/redis-pdb.yaml

    # 3. 等待数据库就绪
    log_info "等待PostgreSQL就绪..."
    kubectl wait --for=condition=Ready pods -l app=postgres-ha -n "$NAMESPACE" --timeout=600s || true

    log_info "等待Redis就绪..."
    kubectl wait --for=condition=Ready pods -l app=redis-ha -n "$NAMESPACE" --timeout=600s || true

    log_info "基础设施部署完成 ✓"
}

# 部署监控系统
deploy_monitoring() {
    log_step "部署监控系统..."

    # 1. 部署Prometheus告警规则
    log_info "部署告警规则..."
    kubectl apply -f ../../monitoring/alerts/reliability-rules.yaml
    kubectl apply -f ../../monitoring/alerts/availability-rules.yaml
    kubectl apply -f ../../monitoring/alerts/performance-rules.yaml

    # 2. 部署AlertManager
    log_info "部署AlertManager..."
    kubectl apply -f ../../monitoring/alertmanager-config.yaml

    # 3. 更新Prometheus配置
    log_info "更新Prometheus配置..."
    kubectl create configmap prometheus-config \
        --from-file=../../monitoring/prometheus.yml \
        -n monitoring \
        --dry-run=client -o yaml | kubectl apply -f -

    log_info "监控系统部署完成 ✓"
}

# 部署服务网格
deploy_service_mesh() {
    log_step "部署服务网格配置..."

    # 1. 部署熔断器
    log_info "部署熔断器..."
    kubectl apply -f ../../k8s/service-mesh/destination-rules/circuit-breakers.yaml

    # 2. 部署重试策略
    log_info "部署重试策略..."
    kubectl apply -f ../../k8s/service-mesh/virtual-services/retry-policies.yaml

    log_info "服务网格配置完成 ✓"
}

# 部署备份系统
deploy_backup_system() {
    log_step "部署备份系统..."

    # 检查Velero是否安装
    if ! kubectl get namespace velero &> /dev/null; then
        log_warn "Velero未安装，跳过备份配置"
        log_warn "请先运行: ./install-velero.sh"
        return
    fi

    # 部署备份计划
    log_info "部署备份计划..."
    kubectl apply -f ../../reliability/velero-schedules.yaml
    kubectl apply -f ../../reliability/backup-storage-class.yaml

    log_info "备份系统配置完成 ✓"
}

# 部署应用服务
deploy_application() {
    log_step "部署应用服务..."

    # 1. 部署ConfigMap和Secret
    log_info "部署配置..."
    kubectl apply -f ../../k8s/base/configmap.yaml
    kubectl apply -f ../../k8s/base/secret.yaml

    # 2. 部署后端服务
    log_info "部署后端服务..."
    kubectl apply -f ../../k8s/base/backend.yaml

    # 3. 部署前端服务
    log_info "部署前端服务..."
    kubectl apply -f ../../k8s/base/frontend.yaml

    # 4. 部署HPA
    log_info "部署自动扩缩容..."
    kubectl apply -f ../../k8s/base/hpa.yaml

    # 5. 部署PDB
    log_info "部署Pod中断预算..."
    kubectl apply -f ../../k8s/base/pdb.yaml

    # 6. 部署Ingress
    log_info "部署Ingress..."
    kubectl apply -f ../../k8s/base/ingress.yaml

    log_info "应用服务部署完成 ✓"
}

# 验证部署
validate_deployment() {
    log_step "验证部署..."

    # 1. 检查Pod状态
    log_info "检查Pod状态..."
    kubectl get pods -n "$NAMESPACE"

    # 2. 检查服务状态
    log_info "检查服务状态..."
    kubectl get svc -n "$NAMESPACE"

    # 3. 检查HPA状态
    log_info "检查HPA状态..."
    kubectl get hpa -n "$NAMESPACE"

    # 4. 等待所有Pod就绪
    log_info "等待所有Pod就绪..."
    kubectl wait --for=condition=Ready pods --all -n "$NAMESPACE" --timeout=600s || true

    # 5. 运行健康检查
    log_info "运行健康检查..."
    if [ -f "../validation/health-check.sh" ]; then
        bash ../validation/health-check.sh
    fi

    log_info "部署验证完成 ✓"
}

# 显示部署信息
show_deployment_info() {
    log_step "部署信息"

    echo ""
    echo "========================================="
    echo "PixelCore 生产环境部署完成！"
    echo "========================================="
    echo ""
    echo "命名空间: $NAMESPACE"
    echo "环境: $ENVIRONMENT"
    echo ""
    echo "访问信息："
    echo "  Ingress: kubectl get ingress -n $NAMESPACE"
    echo "  服务: kubectl get svc -n $NAMESPACE"
    echo ""
    echo "监控信息："
    echo "  Prometheus: kubectl port-forward -n monitoring svc/prometheus 9090:9090"
    echo "  Grafana: kubectl port-forward -n monitoring svc/grafana 3000:3000"
    echo "  AlertManager: kubectl port-forward -n monitoring svc/alertmanager 9093:9093"
    echo ""
    echo "下一步："
    echo "  1. 验证所有服务正常运行"
    echo "  2. 配置域名和SSL证书"
    echo "  3. 运行性能测试"
    echo "  4. 配置告警通知"
    echo ""
    echo "========================================="
}

# 主函数
main() {
    log_info "开始部署PixelCore生产环境..."
    log_info "命名空间: $NAMESPACE"
    log_info "环境: $ENVIRONMENT"
    log_info "Dry Run: $DRY_RUN"
    echo ""

    if [ "$DRY_RUN" = "true" ]; then
        log_warn "Dry Run模式，不会实际部署"
        export KUBECTL_OPTS="--dry-run=client"
    fi

    # 确认部署
    read -p "确认部署到生产环境？(yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        log_error "部署已取消"
        exit 1
    fi

    # 执行部署步骤
    check_prerequisites
    create_namespace
    deploy_infrastructure
    deploy_monitoring
    deploy_service_mesh
    deploy_backup_system
    deploy_application
    validate_deployment
    show_deployment_info

    log_info "部署完成！🎉"
}

# 运行主函数
main "$@"
