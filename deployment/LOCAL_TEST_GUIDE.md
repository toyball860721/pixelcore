# 本地测试环境快速开始

## 前置条件

- Docker Desktop 已安装并运行
- kubectl 已安装
- (可选) Kind 已安装

## 快速开始

### 步骤 1: 创建本地Kubernetes集群

```bash
cd deployment/scripts
./setup-local-test.sh
```

这将：
- 检查Docker是否运行
- 安装Kind（如果未安装）
- 创建3节点Kind集群
- 安装metrics-server
- 创建必需的命名空间
- 创建测试Secret

### 步骤 2: 部署应用到测试环境

```bash
./deploy-test.sh
```

这将部署：
- PostgreSQL（单副本）
- Redis（单副本）
- 后端服务
- 前端服务
- HPA和PDB
- Ingress

### 步骤 3: 验证部署

```bash
# 查看所有Pod
kubectl get pods -n pixelcore

# 运行健康检查
cd ../validation
./health-check.sh
```

### 步骤 4: 访问应用

```bash
# 后端服务
kubectl port-forward -n pixelcore svc/backend-service 8080:8080

# 前端服务
kubectl port-forward -n pixelcore svc/frontend-service 3000:3000
```

然后访问：
- 后端: http://localhost:8080
- 前端: http://localhost:3000

## 常用命令

### 查看资源

```bash
# 查看所有资源
kubectl get all -n pixelcore

# 查看Pod详情
kubectl describe pod <pod-name> -n pixelcore

# 查看日志
kubectl logs -f <pod-name> -n pixelcore

# 查看事件
kubectl get events -n pixelcore --sort-by='.lastTimestamp'
```

### 调试

```bash
# 进入Pod
kubectl exec -it <pod-name> -n pixelcore -- /bin/bash

# 查看Pod资源使用
kubectl top pods -n pixelcore

# 查看节点资源使用
kubectl top nodes
```

### 清理

```bash
# 删除应用
kubectl delete namespace pixelcore

# 删除集群
kind delete cluster --name pixelcore-test
```

## 故障排查

### Pod无法启动

```bash
# 查看Pod状态
kubectl get pods -n pixelcore

# 查看Pod详情
kubectl describe pod <pod-name> -n pixelcore

# 查看日志
kubectl logs <pod-name> -n pixelcore
```

### 镜像拉取失败

Kind集群使用本地Docker镜像。如果镜像不存在：

```bash
# 构建镜像
docker build -t pixelcore/backend:latest ./backend
docker build -t pixelcore/frontend:latest ./frontend

# 加载到Kind
kind load docker-image pixelcore/backend:latest --name pixelcore-test
kind load docker-image pixelcore/frontend:latest --name pixelcore-test
```

### 服务无法访问

```bash
# 检查服务
kubectl get svc -n pixelcore

# 检查端点
kubectl get endpoints -n pixelcore

# 测试服务连接
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
  curl -v http://backend-service.pixelcore:8080/health
```

## 下一步

1. **测试功能**
   - 测试API端点
   - 测试数据库连接
   - 测试Redis缓存

2. **性能测试**
   - 运行负载测试
   - 验证HPA自动扩缩容

3. **可靠性测试**
   - 测试Pod重启
   - 测试故障恢复

4. **准备生产部署**
   - 验证所有功能正常
   - 修复发现的问题
   - 准备生产环境配置

## 参考文档

- [部署指南](../DEPLOYMENT_GUIDE.md)
- [健康检查](../validation/health-check.sh)
- [Kind文档](https://kind.sigs.k8s.io/)
