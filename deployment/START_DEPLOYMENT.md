# 🚀 开始部署 - 详细步骤

## 当前状态检查

✅ kubectl 已安装: `/usr/local/bin/kubectl`
❌ Docker 未运行
❌ Kind 未安装

---

## 📋 需要完成的步骤

### 步骤 1: 启动 Docker Desktop

**你需要做的：**

1. **打开 Docker Desktop 应用**
   - 在 macOS 上：打开 Applications 文件夹，双击 Docker.app
   - 或使用 Spotlight：按 `Cmd + Space`，输入 "Docker"

2. **等待 Docker 启动**
   - 看到菜单栏上的 Docker 图标
   - 图标不再显示动画，表示已启动完成

3. **验证 Docker 运行**
   ```bash
   docker ps
   ```
   应该能看到输出（即使是空的也没关系）

**如果你没有安装 Docker Desktop：**

下载并安装：
- 访问: https://www.docker.com/products/docker-desktop/
- 下载 macOS 版本（Apple Silicon 或 Intel）
- 安装并启动

---

### 步骤 2: 安装 Kind

**Docker 启动后，运行：**

```bash
# 使用 Homebrew 安装 Kind
brew install kind

# 或者手动下载
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-darwin-arm64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind
```

**验证安装：**
```bash
kind version
```

---

### 步骤 3: 创建本地 Kubernetes 集群

**运行我们的脚本：**

```bash
cd deployment/scripts
./setup-local-test.sh
```

**这个脚本会：**
- ✓ 检查 Docker 是否运行
- ✓ 检查 Kind 是否安装（如果没有会尝试安装）
- ✓ 创建 3 节点 Kind 集群
- ✓ 安装 metrics-server
- ✓ 创建命名空间（pixelcore, monitoring, velero）
- ✓ 创建测试 Secret

**预计时间：** 3-5 分钟

---

### 步骤 4: 部署应用

**集群创建成功后：**

```bash
./deploy-test.sh
```

**这会部署：**
- PostgreSQL（单副本）
- Redis（单副本）
- 后端服务
- 前端服务
- HPA 和 PDB
- Ingress

**预计时间：** 2-3 分钟

---

### 步骤 5: 验证部署

```bash
# 查看所有 Pod
kubectl get pods -n pixelcore

# 运行健康检查
cd ../validation
./health-check.sh
```

---

## 🎯 快速命令参考

### 一键完成（Docker 已运行的情况下）

```bash
# 进入脚本目录
cd deployment/scripts

# 安装 Kind（如果需要）
brew install kind

# 创建集群
./setup-local-test.sh

# 部署应用
./deploy-test.sh

# 验证
cd ../validation
./health-check.sh
```

---

## 📊 预期输出

### setup-local-test.sh 成功后：

```
=========================================
本地测试环境已就绪！
=========================================

集群名称: pixelcore-test
节点数量: 4

命名空间:
pixelcore     Active   1m
monitoring    Active   1m
velero        Active   1m

节点状态:
NAME                          STATUS   ROLES           AGE
pixelcore-test-control-plane  Ready    control-plane   2m
pixelcore-test-worker         Ready    <none>          2m
pixelcore-test-worker2        Ready    <none>          2m
pixelcore-test-worker3        Ready    <none>          2m

下一步：
  1. 部署应用: cd deployment/scripts && ./deploy-production.sh
  2. 查看Pod: kubectl get pods -n pixelcore
=========================================
```

### deploy-test.sh 成功后：

```
=========================================
测试环境部署完成！
=========================================

Pods:
NAME                        READY   STATUS    RESTARTS   AGE
backend-xxx                 1/1     Running   0          2m
frontend-xxx                1/1     Running   0          2m
postgres-0                  1/1     Running   0          3m
redis-0                     1/1     Running   0          3m

访问应用：
  kubectl port-forward -n pixelcore svc/backend-service 8080:8080
  kubectl port-forward -n pixelcore svc/frontend-service 3000:3000
=========================================
```

---

## ⚠️ 常见问题

### 问题 1: Docker 未运行

**错误信息：**
```
Cannot connect to the Docker daemon
```

**解决方法：**
1. 打开 Docker Desktop
2. 等待启动完成
3. 重新运行脚本

### 问题 2: Kind 安装失败

**手动安装：**
```bash
# Apple Silicon (M1/M2)
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-darwin-arm64

# Intel Mac
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-darwin-amd64

chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind
```

### 问题 3: 集群创建失败

**清理并重试：**
```bash
# 删除现有集群
kind delete cluster --name pixelcore-test

# 重新创建
./setup-local-test.sh
```

### 问题 4: Pod 无法启动

**查看详情：**
```bash
kubectl get pods -n pixelcore
kubectl describe pod <pod-name> -n pixelcore
kubectl logs <pod-name> -n pixelcore
```

---

## 📞 需要帮助？

如果遇到问题：

1. **查看错误信息**
   ```bash
   kubectl get events -n pixelcore --sort-by='.lastTimestamp'
   ```

2. **查看 Pod 日志**
   ```bash
   kubectl logs -f <pod-name> -n pixelcore
   ```

3. **运行健康检查**
   ```bash
   ./deployment/validation/health-check.sh
   ```

---

## 🎯 现在开始！

**请按照以下顺序执行：**

1. ✅ 启动 Docker Desktop
2. ✅ 安装 Kind: `brew install kind`
3. ✅ 创建集群: `cd deployment/scripts && ./setup-local-test.sh`
4. ✅ 部署应用: `./deploy-test.sh`
5. ✅ 验证: `cd ../validation && ./health-check.sh`

---

**准备好了吗？让我知道你完成了哪一步，或者遇到了什么问题！** 🚀
