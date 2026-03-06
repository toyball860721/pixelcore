#!/bin/bash
# 生产环境回滚脚本

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

NAMESPACE="${NAMESPACE:-pixelcore}"

echo "========================================="
echo "PixelCore 生产环境回滚"
echo "========================================="
echo ""

log_warn "⚠️  警告：这将回滚生产环境！"
echo ""

# 显示当前部署
show_current_deployment() {
    log_info "当前部署状态："
    echo ""
    kubectl get deployments -n "$NAMESPACE"
    echo ""
}

# 回滚选项
rollback_menu() {
    echo "请选择回滚方式："
    echo "  1) 回滚所有服务到上一个版本"
    echo "  2) 回滚特定服务"
    echo "  3) 从Velero备份恢复"
    echo "  4) 取消"
    echo ""
    read -p "选择 (1-4): " choice

    case $choice in
        1)
            rollback_all_services
            ;;
        2)
            rollback_specific_service
            ;;
        3)
            restore_from_backup
            ;;
        4)
            log_info "已取消"
            exit 0
            ;;
        *)
            log_error "无效选择"
            exit 1
            ;;
    esac
}

# 回滚所有服务
rollback_all_services() {
    log_warn "回滚所有服务到上一个版本..."

    read -p "确认回滚所有服务？(yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        log_info "已取消"
        exit 0
    fi

    # 获取所有deployment
    deployments=$(kubectl get deployments -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}')

    for deploy in $deployments; do
        log_info "回滚: $deploy"
        kubectl rollout undo deployment/"$deploy" -n "$NAMESPACE"
    done

    # 等待回滚完成
    log_info "等待回滚完成..."
    for deploy in $deployments; do
        kubectl rollout status deployment/"$deploy" -n "$NAMESPACE" --timeout=300s
    done

    log_info "所有服务回滚完成 ✓"
}

# 回滚特定服务
rollback_specific_service() {
    log_info "可用的服务："
    kubectl get deployments -n "$NAMESPACE" -o name

    echo ""
    read -p "输入要回滚的服务名称: " service_name

    if ! kubectl get deployment "$service_name" -n "$NAMESPACE" &> /dev/null; then
        log_error "服务不存在: $service_name"
        exit 1
    fi

    # 显示历史版本
    log_info "部署历史："
    kubectl rollout history deployment/"$service_name" -n "$NAMESPACE"

    echo ""
    read -p "回滚到哪个版本？(留空回滚到上一个版本): " revision

    if [ -z "$revision" ]; then
        log_info "回滚到上一个版本..."
        kubectl rollout undo deployment/"$service_name" -n "$NAMESPACE"
    else
        log_info "回滚到版本 $revision..."
        kubectl rollout undo deployment/"$service_name" -n "$NAMESPACE" --to-revision="$revision"
    fi

    # 等待回滚完成
    kubectl rollout status deployment/"$service_name" -n "$NAMESPACE" --timeout=300s

    log_info "服务回滚完成 ✓"
}

# 从备份恢复
restore_from_backup() {
    log_warn "从Velero备份恢复..."

    if ! command -v velero &> /dev/null; then
        log_error "velero CLI 未安装"
        exit 1
    fi

    # 列出可用备份
    log_info "可用的备份："
    velero backup get

    echo ""
    read -p "输入要恢复的备份名称: " backup_name

    if [ -z "$backup_name" ]; then
        log_error "备份名称不能为空"
        exit 1
    fi

    # 确认恢复
    log_warn "⚠️  这将删除当前命名空间并从备份恢复！"
    read -p "确认恢复？(yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        log_info "已取消"
        exit 0
    fi

    # 删除当前命名空间
    log_info "删除当前命名空间..."
    kubectl delete namespace "$NAMESPACE" --wait=true

    # 从备份恢复
    log_info "从备份恢复: $backup_name"
    velero restore create restore-$(date +%Y%m%d-%H%M%S) \
        --from-backup "$backup_name" \
        --wait

    # 等待Pod就绪
    log_info "等待Pod就绪..."
    kubectl wait --for=condition=Ready pods --all -n "$NAMESPACE" --timeout=600s || true

    log_info "从备份恢复完成 ✓"
}

# 验证回滚
validate_rollback() {
    log_info "验证回滚..."

    # 检查Pod状态
    kubectl get pods -n "$NAMESPACE"

    # 运行健康检查
    if [ -f "./health-check.sh" ]; then
        bash ./health-check.sh
    fi
}

# 主函数
main() {
    show_current_deployment
    rollback_menu
    validate_rollback

    log_info "回滚完成！"
}

main "$@"
