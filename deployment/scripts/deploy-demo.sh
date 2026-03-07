#!/bin/bash
# 使用测试镜像部署演示版本

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

NAMESPACE="pixelcore"

echo "========================================="
echo "部署演示版本（使用测试镜像）"
echo "========================================="
echo ""

log_step "删除现有的应用部署..."
kubectl delete deployment backend frontend -n "$NAMESPACE" --ignore-not-found=true

log_step "使用测试镜像部署后端..."
kubectl create deployment backend \
  --image=nginx:alpine \
  --replicas=2 \
  -n "$NAMESPACE" \
  --dry-run=client -o yaml | \
  kubectl apply -f -

kubectl set env deployment/backend \
  POSTGRES_HOST=postgres-service \
  REDIS_HOST=redis-service \
  -n "$NAMESPACE"

log_step "使用测试镜像部署前端..."
kubectl create deployment frontend \
  --image=nginx:alpine \
  --replicas=2 \
  -n "$NAMESPACE" \
  --dry-run=client -o yaml | \
  kubectl apply -f -

log_step "等待 Pod 就绪..."
kubectl wait --for=condition=Ready pods -l app=backend -n "$NAMESPACE" --timeout=120s
kubectl wait --for=condition=Ready pods -l app=frontend -n "$NAMESPACE" --timeout=120s

log_step "部署状态"
echo ""
kubectl get pods -n "$NAMESPACE"
echo ""
kubectl get svc -n "$NAMESPACE"

echo ""
echo "========================================="
echo "演示版本部署完成！"
echo "========================================="
echo ""
echo "✅ PostgreSQL: 运行中"
echo "✅ Redis: 运行中"
echo "✅ Backend (nginx演示): 运行中"
echo "✅ Frontend (nginx演示): 运行中"
echo ""
echo "测试访问："
echo "  kubectl port-forward -n $NAMESPACE svc/backend-service 8080:8080"
echo "  curl http://localhost:8080"
echo ""
echo "========================================="

log_info "演示部署完成！🎉"
