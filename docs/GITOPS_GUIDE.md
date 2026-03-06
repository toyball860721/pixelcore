# PixelCore GitOps 指南

## 概述

PixelCore 使用 GitOps 方法进行应用程序和基础设施的声明式管理。通过 ArgoCD，我们实现了自动化部署、版本控制和审计追踪。

## GitOps 原则

1. **声明式配置**: 所有配置都以声明式方式存储在 Git 中
2. **版本控制**: Git 是唯一的真实来源
3. **自动同步**: 自动检测并应用 Git 中的更改
4. **持续协调**: 确保集群状态与 Git 中的期望状态一致

## 架构

### 组件

1. **ArgoCD Server**: Web UI 和 API 服务器
2. **Application Controller**: 监控应用程序并同步状态
3. **Repo Server**: 从 Git 仓库获取配置
4. **Redis**: 缓存和会话存储
5. **Dex**: SSO 和身份验证（可选）

### 工作流程

```
Git Repository → ArgoCD → Kubernetes Cluster
     ↓              ↓              ↓
  Commit        Detect         Apply
  Changes       Drift          Changes
```

## 安装

### 前置条件

- Kubernetes 集群 (v1.24+)
- kubectl 已配置
- Git 仓库访问权限

### 安装步骤

1. **安装 ArgoCD**

```bash
cd k8s/gitops
chmod +x install-argocd.sh
./install-argocd.sh
```

2. **访问 ArgoCD UI**

```bash
kubectl port-forward svc/argocd-server -n argocd 8080:443
```

访问 https://localhost:8080

3. **获取初始密码**

```bash
kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d
```

4. **登录 CLI**

```bash
argocd login localhost:8080 --username admin --password <password> --insecure
```

## 项目配置

### 创建 ArgoCD 项目

```bash
kubectl apply -f argocd/projects.yaml
```

这将创建：
- `pixelcore` 项目
- `pixelcore-dev` 命名空间
- `pixelcore-staging` 命名空间
- `pixelcore` 命名空间（生产）

### 项目权限

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: pixelcore
spec:
  sourceRepos:
  - 'https://github.com/toyball860721/pixelcore.git'
  destinations:
  - namespace: 'pixelcore*'
    server: 'https://kubernetes.default.svc'
  clusterResourceWhitelist:
  - group: '*'
    kind: '*'
```

## 应用程序部署

### 部署所有应用

```bash
kubectl apply -f argocd/application.yaml
```

这将部署：
- pixelcore-api
- pixelcore-search
- pixelcore-ai
- pixelcore-analytics

### 单独部署应用

```bash
# 仅部署 API 服务
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: pixelcore-api
  namespace: argocd
spec:
  project: pixelcore
  source:
    repoURL: https://github.com/toyball860721/pixelcore.git
    targetRevision: HEAD
    path: k8s/gitops/apps/pixelcore-api
  destination:
    server: https://kubernetes.default.svc
    namespace: pixelcore
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
EOF
```

### 查看应用状态

```bash
# CLI
argocd app list
argocd app get pixelcore-api

# kubectl
kubectl get applications -n argocd
```

## 多环境管理

### 环境配置

PixelCore 支持三个环境：

1. **Development (dev)**
   - 命名空间: `pixelcore-dev`
   - 副本数: 1
   - 镜像标签: `dev-latest`
   - 自动同步: 启用

2. **Staging**
   - 命名空间: `pixelcore-staging`
   - 副本数: 2
   - 镜像标签: `staging-v1.0.0`
   - 自动同步: 启用

3. **Production**
   - 命名空间: `pixelcore`
   - 副本数: 5
   - 镜像标签: `v1.0.0`
   - 自动同步: 需要审批

### 使用 Kustomize 管理环境

**开发环境：**
```yaml
# environments/dev/kustomization.yaml
bases:
- ../../apps/pixelcore-api

namespace: pixelcore-dev
namePrefix: dev-

replicas:
- name: api
  count: 1

images:
- name: pixelcore/api
  newTag: dev-latest
```

**生产环境：**
```yaml
# environments/production/kustomization.yaml
bases:
- ../../apps/pixelcore-api

namespace: pixelcore

replicas:
- name: api
  count: 5

images:
- name: pixelcore/api
  newTag: v1.0.0
```

## 同步策略

### 自动同步

启用自动同步后，ArgoCD 会自动应用 Git 中的更改：

```yaml
syncPolicy:
  automated:
    prune: true        # 删除不在 Git 中的资源
    selfHeal: true     # 自动修复漂移
    allowEmpty: false  # 不允许空应用
```

### 手动同步

对于生产环境，建议使用手动同步：

```bash
# CLI
argocd app sync pixelcore-api

# UI
点击 "SYNC" 按钮
```

### 同步选项

```yaml
syncOptions:
- CreateNamespace=true    # 自动创建命名空间
- PruneLast=true          # 最后删除资源
- ApplyOutOfSyncOnly=true # 仅应用不同步的资源
```

## 回滚

### 查看历史

```bash
argocd app history pixelcore-api
```

### 回滚到特定版本

```bash
# 回滚到上一个版本
argocd app rollback pixelcore-api

# 回滚到特定版本
argocd app rollback pixelcore-api 5
```

### Git 回滚

```bash
# 回滚 Git 提交
git revert <commit-hash>
git push

# ArgoCD 会自动检测并应用更改
```

## 健康检查

### 应用健康状态

ArgoCD 自动检测应用健康状态：

- **Healthy**: 所有资源正常运行
- **Progressing**: 正在部署
- **Degraded**: 部分资源失败
- **Suspended**: 应用已暂停
- **Missing**: 资源缺失
- **Unknown**: 状态未知

### 自定义健康检查

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-cm
  namespace: argocd
data:
  resource.customizations: |
    apps/Deployment:
      health.lua: |
        hs = {}
        if obj.status ~= nil then
          if obj.status.replicas ~= nil and obj.status.updatedReplicas ~= nil then
            if obj.status.replicas == obj.status.updatedReplicas then
              hs.status = "Healthy"
              hs.message = "All replicas are updated"
              return hs
            end
          end
        end
        hs.status = "Progressing"
        hs.message = "Waiting for rollout to finish"
        return hs
```

## 通知

### Slack 通知

配置 Slack 通知：

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-notifications-cm
  namespace: argocd
data:
  service.slack: |
    token: $slack-token
  template.app-deployed: |
    message: |
      Application {{.app.metadata.name}} deployed successfully!
    slack:
      attachments: |
        [{
          "title": "{{ .app.metadata.name}}",
          "color": "#18be52",
          "fields": [
            {
              "title": "Sync Status",
              "value": "{{.app.status.sync.status}}"
            }
          ]
        }]
```

### 触发器

```yaml
trigger.on-deployed: |
  - when: app.status.operationState.phase in ['Succeeded']
    send: [app-deployed]
  - when: app.status.operationState.phase in ['Error', 'Failed']
    send: [app-sync-failed]
```

## CI/CD 集成

### GitHub Actions

```yaml
name: Deploy to ArgoCD

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2

    - name: Update image tag
      run: |
        cd k8s/gitops/apps/pixelcore-api
        kustomize edit set image pixelcore/api:${{ github.sha }}

    - name: Commit and push
      run: |
        git config user.name "GitHub Actions"
        git config user.email "actions@github.com"
        git add .
        git commit -m "Update image to ${{ github.sha }}"
        git push
```

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - cd k8s/gitops/apps/pixelcore-api
    - kustomize edit set image pixelcore/api:$CI_COMMIT_SHA
    - git add .
    - git commit -m "Update image to $CI_COMMIT_SHA"
    - git push
  only:
    - main
```

## RBAC

### 创建只读用户

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-rbac-cm
  namespace: argocd
data:
  policy.csv: |
    p, role:readonly, applications, get, */*, allow
    p, role:readonly, applications, list, */*, allow
    g, readonly-user, role:readonly
```

### 创建部署用户

```yaml
policy.csv: |
  p, role:deployer, applications, sync, pixelcore/*, allow
  p, role:deployer, applications, get, pixelcore/*, allow
  g, deployer-user, role:deployer
```

## 最佳实践

### 1. 使用 Git 分支策略

```
main (production)
  ↑
staging
  ↑
develop
```

### 2. 环境隔离

- 每个环境使用独立的命名空间
- 使用 Kustomize 管理环境差异
- 生产环境需要审批

### 3. 版本管理

- 使用语义化版本 (v1.0.0)
- 开发环境使用 `dev-latest`
- 生产环境使用固定版本

### 4. 安全性

- 启用 RBAC
- 使用 SSO 集成
- 定期审计权限
- 加密敏感数据

### 5. 监控

- 监控同步状态
- 设置告警
- 定期检查健康状态
- 审计部署历史

## 故障排查

### 同步失败

**检查应用状态：**
```bash
argocd app get pixelcore-api
kubectl describe application pixelcore-api -n argocd
```

**查看日志：**
```bash
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller
```

### 健康检查失败

**检查资源状态：**
```bash
kubectl get pods -n pixelcore
kubectl describe pod <pod-name> -n pixelcore
```

### Git 同步问题

**检查仓库连接：**
```bash
argocd repo list
argocd repo get https://github.com/toyball860721/pixelcore.git
```

## 性能优化

### 1. 减少同步频率

```yaml
spec:
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

### 2. 使用应用集

对于大量相似应用，使用 ApplicationSet：

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: pixelcore-services
spec:
  generators:
  - list:
      elements:
      - name: api
      - name: search
      - name: ai
  template:
    metadata:
      name: 'pixelcore-{{name}}'
    spec:
      project: pixelcore
      source:
        repoURL: https://github.com/toyball860721/pixelcore.git
        path: 'k8s/gitops/apps/pixelcore-{{name}}'
```

### 3. 缓存优化

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-cm
  namespace: argocd
data:
  timeout.reconciliation: 180s
  timeout.hard.reconciliation: 0s
```

## 监控指标

### 关键指标

1. **同步状态**
   - `argocd_app_sync_total`
   - 成功/失败的同步次数

2. **健康状态**
   - `argocd_app_health_status`
   - 应用健康状态

3. **同步延迟**
   - `argocd_app_reconcile_duration_seconds`
   - 同步耗时

4. **Git 操作**
   - `argocd_git_request_total`
   - Git 请求次数

### Prometheus 集成

```yaml
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: argocd-metrics
  namespace: argocd
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: argocd-metrics
  endpoints:
  - port: metrics
```

## 安全加固

### 1. 启用 SSO

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-cm
  namespace: argocd
data:
  url: https://argocd.pixelcore.io
  dex.config: |
    connectors:
    - type: github
      id: github
      name: GitHub
      config:
        clientID: $github-client-id
        clientSecret: $github-client-secret
```

### 2. 加密敏感数据

使用 Sealed Secrets 或 External Secrets：

```bash
# 安装 Sealed Secrets
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.18.0/controller.yaml

# 创建加密的 Secret
kubeseal --format yaml < secret.yaml > sealed-secret.yaml
```

### 3. 网络策略

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: argocd-network-policy
  namespace: argocd
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: argocd-server
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: istio-system
    ports:
    - protocol: TCP
      port: 8080
```

## 参考资源

- [ArgoCD 官方文档](https://argo-cd.readthedocs.io/)
- [Kustomize 文档](https://kustomize.io/)
- [GitOps 原则](https://www.gitops.tech/)
- [Kubernetes 最佳实践](https://kubernetes.io/docs/concepts/configuration/overview/)

---

**最后更新**: 2026-03-06
**版本**: 1.0.0
**ArgoCD 版本**: 2.9.0
