#!/bin/bash
# 生产环境健康检查脚本

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

NAMESPACE="${NAMESPACE:-pixelcore}"
FAILED_CHECKS=0

echo "========================================="
echo "PixelCore 生产环境健康检查"
echo "========================================="
echo ""

# 1. 检查命名空间
check_namespace() {
    echo "1. 检查命名空间..."
    if kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_info "命名空间存在: $NAMESPACE"
    else
        log_error "命名空间不存在: $NAMESPACE"
        ((FAILED_CHECKS++))
    fi
}

# 2. 检查Pod状态
check_pods() {
    echo ""
    echo "2. 检查Pod状态..."

    local total_pods=$(kubectl get pods -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)
    local running_pods=$(kubectl get pods -n "$NAMESPACE" --no-headers 2>/dev/null | grep -c Running || echo 0)

    if [ "$total_pods" -eq 0 ]; then
        log_error "没有找到Pod"
        ((FAILED_CHECKS++))
        return
    fi

    if [ "$running_pods" -eq "$total_pods" ]; then
        log_info "所有Pod运行正常: $running_pods/$total_pods"
    else
        log_warn "部分Pod未运行: $running_pods/$total_pods"
        kubectl get pods -n "$NAMESPACE" | grep -v Running || true
        ((FAILED_CHECKS++))
    fi
}

# 3. 检查数据库
check_database() {
    echo ""
    echo "3. 检查PostgreSQL..."

    if kubectl get statefulset postgres-ha -n "$NAMESPACE" &> /dev/null; then
        local replicas=$(kubectl get statefulset postgres-ha -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
        local ready=$(kubectl get statefulset postgres-ha -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')

        if [ "$ready" = "$replicas" ]; then
            log_info "PostgreSQL HA: $ready/$replicas 副本就绪"

            # 检查复制状态
            if kubectl exec -it postgres-ha-0 -n "$NAMESPACE" -- pg_isready &> /dev/null; then
                log_info "PostgreSQL 主节点健康"
            else
                log_error "PostgreSQL 主节点不健康"
                ((FAILED_CHECKS++))
            fi
        else
            log_error "PostgreSQL HA: 只有 $ready/$replicas 副本就绪"
            ((FAILED_CHECKS++))
        fi
    else
        log_warn "PostgreSQL HA 未部署"
    fi
}

# 4. 检查Redis
check_redis() {
    echo ""
    echo "4. 检查Redis..."

    if kubectl get statefulset redis-ha -n "$NAMESPACE" &> /dev/null; then
        local replicas=$(kubectl get statefulset redis-ha -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
        local ready=$(kubectl get statefulset redis-ha -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')

        if [ "$ready" = "$replicas" ]; then
            log_info "Redis HA: $ready/$replicas 副本就绪"

            # 检查Redis连接
            if kubectl exec -it redis-ha-0 -n "$NAMESPACE" -- redis-cli ping &> /dev/null; then
                log_info "Redis 主节点健康"
            else
                log_error "Redis 主节点不健康"
                ((FAILED_CHECKS++))
            fi
        else
            log_error "Redis HA: 只有 $ready/$replicas 副本就绪"
            ((FAILED_CHECKS++))
        fi
    else
        log_warn "Redis HA 未部署"
    fi

    # 检查Sentinel
    if kubectl get statefulset redis-sentinel -n "$NAMESPACE" &> /dev/null; then
        local sentinel_ready=$(kubectl get statefulset redis-sentinel -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
        log_info "Redis Sentinel: $sentinel_ready 副本就绪"
    fi
}

# 5. 检查服务
check_services() {
    echo ""
    echo "5. 检查服务..."

    local services=("backend-service" "frontend-service" "postgres-ha-service" "redis-ha-service")

    for svc in "${services[@]}"; do
        if kubectl get svc "$svc" -n "$NAMESPACE" &> /dev/null; then
            log_info "服务存在: $svc"
        else
            log_warn "服务不存在: $svc"
        fi
    done
}

# 6. 检查HPA
check_hpa() {
    echo ""
    echo "6. 检查自动扩缩容..."

    local hpa_count=$(kubectl get hpa -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)

    if [ "$hpa_count" -gt 0 ]; then
        log_info "HPA配置: $hpa_count 个"
        kubectl get hpa -n "$NAMESPACE" --no-headers | while read line; do
            echo "  - $line"
        done
    else
        log_warn "未配置HPA"
    fi
}

# 7. 检查PDB
check_pdb() {
    echo ""
    echo "7. 检查Pod中断预算..."

    local pdb_count=$(kubectl get pdb -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)

    if [ "$pdb_count" -gt 0 ]; then
        log_info "PDB配置: $pdb_count 个"
    else
        log_warn "未配置PDB"
    fi
}

# 8. 检查监控
check_monitoring() {
    echo ""
    echo "8. 检查监控系统..."

    # 检查Prometheus
    if kubectl get pods -n monitoring -l app=prometheus 2>/dev/null | grep -q Running; then
        log_info "Prometheus 运行中"
    else
        log_warn "Prometheus 未运行"
    fi

    # 检查AlertManager
    if kubectl get pods -n monitoring -l app=alertmanager 2>/dev/null | grep -q Running; then
        log_info "AlertManager 运行中"
    else
        log_warn "AlertManager 未运行"
    fi
}

# 9. 检查备份
check_backups() {
    echo ""
    echo "9. 检查备份系统..."

    if command -v velero &> /dev/null; then
        if kubectl get namespace velero &> /dev/null; then
            log_info "Velero 已安装"

            local backup_count=$(velero backup get 2>/dev/null | grep -c Completed || echo 0)
            if [ "$backup_count" -gt 0 ]; then
                log_info "备份数量: $backup_count"
            else
                log_warn "没有完成的备份"
            fi
        else
            log_warn "Velero 未安装"
        fi
    else
        log_warn "velero CLI 未安装"
    fi
}

# 10. 检查资源使用
check_resources() {
    echo ""
    echo "10. 检查资源使用..."

    echo ""
    echo "节点资源:"
    kubectl top nodes 2>/dev/null || log_warn "无法获取节点资源（需要metrics-server）"

    echo ""
    echo "Pod资源 (Top 5):"
    kubectl top pods -n "$NAMESPACE" 2>/dev/null | head -6 || log_warn "无法获取Pod资源"
}

# 运行所有检查
main() {
    check_namespace
    check_pods
    check_database
    check_redis
    check_services
    check_hpa
    check_pdb
    check_monitoring
    check_backups
    check_resources

    echo ""
    echo "========================================="
    if [ $FAILED_CHECKS -eq 0 ]; then
        log_info "所有检查通过！系统健康 ✓"
        echo "========================================="
        exit 0
    else
        log_error "发现 $FAILED_CHECKS 个问题"
        echo "========================================="
        exit 1
    fi
}

main "$@"
