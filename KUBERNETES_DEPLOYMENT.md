# Kubernetes 部署指南

本文档介绍如何在 Kubernetes 集群上部署 PixelCore。

---

## 📋 前置要求

- Kubernetes 集群 1.24+
- kubectl 命令行工具
- Helm 3.0+ (可选)
- Kustomize 4.0+ (可选)
- 至少 8GB 可用内存
- 至少 50GB 可用存储

---

## 🚀 快速开始

### 方法 1: 使用 kubectl

```bash
# 应用所有基础配置
kubectl apply -k k8s/base/

# 或者应用特定环境
kubectl apply -k k8s/overlays/dev/     # 开发环境
kubectl apply -k k8s/overlays/prod/    # 生产环境
```

### 方法 2: 使用 Helm

```bash
# 安装 Chart
helm install pixelcore ./helm/pixelcore \
  --namespace pixelcore \
  --create-namespace

# 或使用自定义值
helm install pixelcore ./helm/pixelcore \
  --namespace pixelcore \
  --create-namespace \
  --values custom-values.yaml
```

---

## 🏗️ 架构说明

### 组件

| 组件 | 类型 | 副本数 | 资源 |
|------|------|--------|------|
| Backend | Deployment | 3-10 (HPA) | 512Mi-1Gi / 0.5-1 CPU |
| Frontend | Deployment | 2-5 (HPA) | 128Mi-256Mi / 0.1-0.2 CPU |
| PostgreSQL | StatefulSet | 1 | 512Mi-1Gi / 0.5-1 CPU |
| Redis | StatefulSet | 1 | 256Mi-512Mi / 0.25-0.5 CPU |

### 网络拓扑

```
┌─────────────────────────────────────────────────────────┐
│                    Ingress Controller                    │
│              (pixelcore.example.com)                     │
│              (api.pixelcore.example.com)                 │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌──────▼──────┐  ┌────────▼────────┐
│  Frontend      │  │  Backend    │  │  Backend        │
│  Pod (1)       │  │  Pod (1)    │  │  Pod (2)        │
└────────────────┘  └─────────────┘  └─────────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌──────▼──────┐
│  PostgreSQL    │  │  Redis      │
│  StatefulSet   │  │  StatefulSet│
└────────────────┘  └─────────────┘
```

---

## 📦 部署步骤

### 1. 准备集群

```bash
# 检查集群状态
kubectl cluster-info
kubectl get nodes

# 创建命名空间
kubectl create namespace pixelcore
```

### 2. 配置 Secrets

```bash
# 创建 Secret
kubectl create secret generic pixelcore-secrets \
  --namespace=pixelcore \
  --from-literal=POSTGRES_PASSWORD='your-secure-password' \
  --from-literal=REDIS_PASSWORD='your-secure-password' \
  --from-literal=DATABASE_URL='postgresql://pixelcore:your-password@postgres-service:5432/pixelcore' \
  --from-literal=REDIS_URL='redis://:your-password@redis-service:6379' \
  --from-literal=SECRET_KEY='your-secret-key' \
  --from-literal=JWT_SECRET='your-jwt-secret'
```

### 3. 部署数据库

```bash
# 部署 PostgreSQL
kubectl apply -f k8s/base/postgres.yaml

# 等待 PostgreSQL 就绪
kubectl wait --for=condition=ready pod -l app=postgres -n pixelcore --timeout=300s

# 部署 Redis
kubectl apply -f k8s/base/redis.yaml

# 等待 Redis 就绪
kubectl wait --for=condition=ready pod -l app=redis -n pixelcore --timeout=300s
```

### 4. 部署应用

```bash
# 部署后端
kubectl apply -f k8s/base/backend.yaml

# 部署前端
kubectl apply -f k8s/base/frontend.yaml

# 等待应用就绪
kubectl wait --for=condition=ready pod -l app=backend -n pixelcore --timeout=300s
kubectl wait --for=condition=ready pod -l app=frontend -n pixelcore --timeout=300s
```

### 5. 配置 Ingress

```bash
# 安装 Nginx Ingress Controller (如果未安装)
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml

# 安装 cert-manager (如果未安装)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# 部署 Ingress
kubectl apply -f k8s/base/ingress.yaml
```

### 6. 配置自动扩展

```bash
# 部署 HPA
kubectl apply -f k8s/base/hpa.yaml

# 部署 PDB
kubectl apply -f k8s/base/pdb.yaml
```

---

## 🔧 配置管理

### 使用 Kustomize

#### 开发环境

```bash
# 预览配置
kubectl kustomize k8s/overlays/dev/

# 应用配置
kubectl apply -k k8s/overlays/dev/
```

#### 生产环境

```bash
# 预览配置
kubectl kustomize k8s/overlays/prod/

# 应用配置
kubectl apply -k k8s/overlays/prod/
```

### 使用 Helm

#### 安装

```bash
helm install pixelcore ./helm/pixelcore \
  --namespace pixelcore \
  --create-namespace \
  --set backend.replicaCount=5 \
  --set postgresql.auth.password=your-password \
  --set redis.auth.password=your-password
```

#### 升级

```bash
helm upgrade pixelcore ./helm/pixelcore \
  --namespace pixelcore \
  --reuse-values \
  --set backend.image.tag=v1.1.0
```

#### 回滚

```bash
# 查看历史
helm history pixelcore -n pixelcore

# 回滚到上一个版本
helm rollback pixelcore -n pixelcore

# 回滚到指定版本
helm rollback pixelcore 2 -n pixelcore
```

---

## 📊 监控和日志

### 查看 Pod 状态

```bash
# 查看所有 Pod
kubectl get pods -n pixelcore

# 查看详细信息
kubectl describe pod <pod-name> -n pixelcore

# 查看日志
kubectl logs <pod-name> -n pixelcore

# 实时查看日志
kubectl logs -f <pod-name> -n pixelcore
```

### 查看服务状态

```bash
# 查看所有服务
kubectl get svc -n pixelcore

# 查看 Ingress
kubectl get ingress -n pixelcore

# 查看 HPA 状态
kubectl get hpa -n pixelcore
```

### 查看资源使用

```bash
# 查看节点资源
kubectl top nodes

# 查看 Pod 资源
kubectl top pods -n pixelcore
```

---

## 🔍 故障排查

### Pod 无法启动

```bash
# 查看 Pod 事件
kubectl describe pod <pod-name> -n pixelcore

# 查看日志
kubectl logs <pod-name> -n pixelcore

# 查看上一个容器的日志
kubectl logs <pod-name> -n pixelcore --previous
```

### 服务无法访问

```bash
# 检查服务
kubectl get svc -n pixelcore

# 检查端点
kubectl get endpoints -n pixelcore

# 测试服务连接
kubectl run -it --rm debug --image=busybox --restart=Never -n pixelcore -- sh
# 在容器内执行
wget -O- http://backend-service:8080/health
```

### 数据库连接失败

```bash
# 检查 PostgreSQL Pod
kubectl get pods -l app=postgres -n pixelcore

# 查看 PostgreSQL 日志
kubectl logs -l app=postgres -n pixelcore

# 进入 PostgreSQL 容器
kubectl exec -it postgres-0 -n pixelcore -- psql -U pixelcore -d pixelcore
```

### HPA 不工作

```bash
# 检查 Metrics Server
kubectl get deployment metrics-server -n kube-system

# 安装 Metrics Server (如果未安装)
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# 查看 HPA 详情
kubectl describe hpa backend-hpa -n pixelcore
```

---

## 🔄 更新和维护

### 滚动更新

```bash
# 更新镜像
kubectl set image deployment/backend backend=ghcr.io/your-org/pixelcore/backend:v1.1.0 -n pixelcore

# 查看更新状态
kubectl rollout status deployment/backend -n pixelcore

# 查看更新历史
kubectl rollout history deployment/backend -n pixelcore
```

### 回滚

```bash
# 回滚到上一个版本
kubectl rollout undo deployment/backend -n pixelcore

# 回滚到指定版本
kubectl rollout undo deployment/backend --to-revision=2 -n pixelcore
```

### 扩缩容

```bash
# 手动扩容
kubectl scale deployment backend --replicas=5 -n pixelcore

# 查看扩容状态
kubectl get deployment backend -n pixelcore
```

---

## 🔒 安全最佳实践

### 1. 使用 Secrets 管理敏感信息

```bash
# 从文件创建 Secret
kubectl create secret generic pixelcore-secrets \
  --from-file=postgres-password=./postgres-password.txt \
  --namespace=pixelcore

# 使用 Sealed Secrets (推荐)
kubeseal --format=yaml < secret.yaml > sealed-secret.yaml
kubectl apply -f sealed-secret.yaml
```

### 2. 配置 RBAC

```bash
# 创建 ServiceAccount
kubectl create serviceaccount pixelcore-sa -n pixelcore

# 创建 Role 和 RoleBinding
kubectl apply -f k8s/rbac.yaml
```

### 3. 配置 Network Policies

```bash
# 应用网络策略
kubectl apply -f k8s/network-policy.yaml
```

### 4. 启用 Pod Security Standards

```bash
# 为命名空间添加标签
kubectl label namespace pixelcore \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/warn=restricted
```

---

## 📈 性能优化

### 1. 资源限制

- 为所有容器设置 requests 和 limits
- 使用 HPA 自动扩展
- 配置 PDB 保证可用性

### 2. 存储优化

- 使用 SSD 存储类
- 配置合理的存储大小
- 启用数据库持久化

### 3. 网络优化

- 使用 Service Mesh (可选)
- 配置 Ingress 缓存
- 启用 HTTP/2

---

## 🌐 多集群部署

### 使用 Cluster Federation

```bash
# 安装 KubeFed
kubectl apply -f https://github.com/kubernetes-sigs/kubefed/releases/download/v0.10.0/kubefed.yaml

# 加入集群
kubefedctl join cluster1 --host-cluster-context=cluster1
kubefedctl join cluster2 --host-cluster-context=cluster2
```

### 使用 GitOps (ArgoCD)

```bash
# 安装 ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# 创建应用
kubectl apply -f argocd-app.yaml
```

---

## 📚 参考资料

- [Kubernetes 官方文档](https://kubernetes.io/docs/)
- [Helm 文档](https://helm.sh/docs/)
- [Kustomize 文档](https://kustomize.io/)
- [Ingress Nginx 文档](https://kubernetes.github.io/ingress-nginx/)
- [cert-manager 文档](https://cert-manager.io/docs/)

---

## 🆘 获取帮助

如果遇到问题:

1. 查看 Pod 日志和事件
2. 查看本文档的故障排查部分
3. 提交 Issue: https://github.com/your-org/pixelcore/issues

---

**部署愉快！** 🚀
