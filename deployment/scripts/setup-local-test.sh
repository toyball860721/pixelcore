#!/bin/bash
# 本地测试环境快速启动脚本
# 使用Kind创建本地Kubernetes集群

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

CLUSTER_NAME="${CLUSTER_NAME:-pixelcore-test}"

echo "========================================="
echo "PixelCore 本地测试环境启动"
echo "========================================="
echo ""

# 检查Docker
check_docker() {
    log_step "检查Docker..."
    if ! command -v docker &> /dev/null; then
        log_error "Docker未安装"
        log_info "请先安装Docker: https://docs.docker.com/get-docker/"
        exit 1
    fi

    if ! docker ps &> /dev/null; then
        log_error "Docker未运行"
        log_info "请启动Docker"
        exit 1
    fi

    log_info "Docker已就绪 ✓"
}

# 检查Kind
check_kind() {
    log_step "检查Kind..."
    if ! command -v kind &> /dev/null; then
        log_warn "Kind未安装，正在安装..."

        # 检测操作系统
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            if command -v brew &> /dev/null; then
                brew install kind
            else
                log_error "请先安装Homebrew或手动安装Kind"
                log_info "安装命令: brew install kind"
                log_info "或访问: https://kind.sigs.k8s.io/docs/user/quick-start/"
                exit 1
            fi
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux
            curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
            chmod +x ./kind
            sudo mv ./kind /usr/local/bin/kind
        else
            log_error "不支持的操作系统: $OSTYPE"
            exit 1
        fi
    fi

    log_info "Kind已就绪 ✓"
}

# 创建Kind集群
create_cluster() {
    log_step "创建Kind集群: $CLUSTER_NAME"

    # 检查集群是否已存在
    if kind get clusters | grep -q "^${CLUSTER_NAME}$"; then
        log_warn "集群已存在: $CLUSTER_NAME"
        read -p "是否删除并重新创建？(yes/no): " confirm
        if [ "$confirm" = "yes" ]; then
            log_info "删除现有集群..."
            kind delete cluster --name "$CLUSTER_NAME"
        else
            log_info "使用现有集群"
            return
        fi
    fi

    # 创建集群配置
    cat > /tmp/kind-config.yaml <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
- role: worker
- role: worker
- role: worker
EOF

    log_info "创建3节点集群（1 control-plane + 3 workers）..."
    kind create cluster --name "$CLUSTER_NAME" --config /tmp/kind-config.yaml

    # 等待集群就绪
    log_info "等待集群就绪..."
    kubectl wait --for=condition=Ready nodes --all --timeout=300s

    log_info "集群创建成功 ✓"
}

# 安装基础组件
install_basics() {
    log_step "安装基础组件..."

    # 安装metrics-server（用于kubectl top）
    log_info "安装metrics-server..."
    kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

    # 修补metrics-server以在Kind中工作
    kubectl patch deployment metrics-server -n kube-system --type='json' \
        -p='[{"op": "add", "path": "/spec/template/spec/containers/0/args/-", "value": "--kubelet-insecure-tls"}]' || true

    log_info "基础组件安装完成 ✓"
}

# 创建命名空间
create_namespaces() {
    log_step "创建命名空间..."

    kubectl create namespace pixelcore --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace velero --dry-run=client -o yaml | kubectl apply -f -

    kubectl label namespace pixelcore environment=test --overwrite
    kubectl label namespace monitoring environment=test --overwrite

    log_info "命名空间创建完成 ✓"
}

# 创建测试Secret
create_test_secrets() {
    log_step "创建测试Secret..."

    # 创建pixelcore secrets
    kubectl create secret generic pixelcore-secrets \
        --from-literal=POSTGRES_PASSWORD=test-password-123 \
        --from-literal=REDIS_PASSWORD=test-password-456 \
        --from-literal=JWT_SECRET=test-jwt-secret-789 \
        -n pixelcore \
        --dry-run=client -o yaml | kubectl apply -f -

    # 创建alertmanager secrets
    kubectl create secret generic alertmanager-secrets \
        --from-literal=slack-webhook-url=https://hooks.slack.com/services/TEST/TEST/TEST \
        --from-literal=smtp-password=test-smtp-password \
        -n monitoring \
        --dry-run=client -o yaml | kubectl apply -f -

    log_info "测试Secret创建完成 ✓"
}

# 显示集群信息
show_cluster_info() {
    log_step "集群信息"

    echo ""
    echo "========================================="
    echo "本地测试环境已就绪！"
    echo "========================================="
    echo ""
    echo "集群名称: $CLUSTER_NAME"
    echo "节点数量: $(kubectl get nodes --no-headers | wc -l)"
    echo ""
    echo "命名空间:"
    kubectl get namespaces | grep -E "pixelcore|monitoring|velero"
    echo ""
    echo "节点状态:"
    kubectl get nodes
    echo ""
    echo "========================================="
    echo ""
    echo "下一步："
    echo "  1. 部署应用: cd deployment/scripts && ./deploy-production.sh"
    echo "  2. 查看Pod: kubectl get pods -n pixelcore"
    echo "  3. 查看日志: kubectl logs -f <pod-name> -n pixelcore"
    echo "  4. 删除集群: kind delete cluster --name $CLUSTER_NAME"
    echo ""
    echo "========================================="
}

# 主函数
main() {
    check_docker
    check_kind
    create_cluster
    install_basics
    create_namespaces
    create_test_secrets
    show_cluster_info

    log_info "本地测试环境启动完成！🎉"
}

main "$@"
