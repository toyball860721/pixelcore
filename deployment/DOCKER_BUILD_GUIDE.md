# Docker 镜像构建指南

## 概述

本指南说明如何构建 PixelCore 的 Docker 镜像，包括 Backend (Rust) 和 Frontend (React)。

## 镜像架构

### Backend 镜像
- **基础镜像 (构建)**: rust:1.75-slim
- **基础镜像 (运行)**: debian:bookworm-slim
- **构建方式**: 多阶段构建
- **暴露端口**: 8080
- **健康检查**: http://localhost:8080/health

### Frontend 镜像
- **基础镜像 (构建)**: node:20-alpine
- **基础镜像 (运行)**: nginx:alpine
- **构建方式**: 多阶段构建
- **暴露端口**: 80
- **健康检查**: http://localhost/health

## 快速开始

### 1. 构建所有镜像

```bash
cd /Users/toyball/Desktop/ClaudeUse/pixelcore
./deployment/scripts/build-images.sh
```

这个脚本会：
- 检查 Docker 是否运行
- 构建 Backend 镜像
- 构建 Frontend 镜像
- 创建本地标签 (pixelcore/backend:latest, pixelcore/frontend:latest)
- 可选：推送到镜像仓库

### 2. 手动构建单个镜像

#### Backend
```bash
cd /Users/toyball/Desktop/ClaudeUse/pixelcore
docker build -t pixelcore/backend:latest -f Dockerfile .
```

#### Frontend
```bash
cd /Users/toyball/Desktop/ClaudeUse/pixelcore/app
docker build -t pixelcore/frontend:latest -f Dockerfile .
```

## 镜像标签

### 本地开发
- `pixelcore/backend:latest`
- `pixelcore/frontend:latest`

### 镜像仓库 (默认)
- `ghcr.io/your-org/pixelcore/backend:latest`
- `ghcr.io/your-org/pixelcore/frontend:latest`

### 自定义标签
```bash
# 使用环境变量自定义
export DOCKER_REGISTRY="your-registry.com/pixelcore"
export VERSION="v1.0.0"
./deployment/scripts/build-images.sh
```

## 测试镜像

### Backend
```bash
# 运行 backend 容器
docker run -d -p 8080:8080 \
  --name pixelcore-backend \
  -e DATABASE_URL="postgres://user:pass@host/db" \
  -e REDIS_URL="redis://host:6379" \
  pixelcore/backend:latest

# 检查健康状态
curl http://localhost:8080/health

# 查看日志
docker logs -f pixelcore-backend

# 停止并删除
docker stop pixelcore-backend
docker rm pixelcore-backend
```

### Frontend
```bash
# 运行 frontend 容器
docker run -d -p 80:80 \
  --name pixelcore-frontend \
  pixelcore/frontend:latest

# 访问应用
open http://localhost

# 查看日志
docker logs -f pixelcore-frontend

# 停止并删除
docker stop pixelcore-frontend
docker rm pixelcore-frontend
```

## 推送到镜像仓库

### GitHub Container Registry (GHCR)

1. **登录 GHCR**
```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
```

2. **推送镜像**
```bash
docker push ghcr.io/your-org/pixelcore/backend:latest
docker push ghcr.io/your-org/pixelcore/frontend:latest
```

### Docker Hub

1. **登录 Docker Hub**
```bash
docker login
```

2. **推送镜像**
```bash
docker tag pixelcore/backend:latest your-username/pixelcore-backend:latest
docker tag pixelcore/frontend:latest your-username/pixelcore-frontend:latest

docker push your-username/pixelcore-backend:latest
docker push your-username/pixelcore-frontend:latest
```

## 部署到 Kubernetes

### 更新部署使用新镜像

```bash
# 使用脚本自动更新
./deployment/scripts/update-deployment-images.sh

# 或手动更新
kubectl set image deployment/backend backend=pixelcore/backend:latest -n pixelcore
kubectl set image deployment/frontend frontend=pixelcore/frontend:latest -n pixelcore

# 等待部署完成
kubectl rollout status deployment/backend -n pixelcore
kubectl rollout status deployment/frontend -n pixelcore
```

### 验证部署

```bash
# 检查 pods
kubectl get pods -n pixelcore

# 检查镜像版本
kubectl get pods -n pixelcore -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[*].image}{"\n"}{end}'

# 运行健康检查
./deployment/scripts/health-check.sh
```

## 构建优化

### 加速构建

1. **使用 Docker BuildKit**
```bash
export DOCKER_BUILDKIT=1
docker build ...
```

2. **使用构建缓存**
```bash
# Backend - 缓存依赖层
docker build --cache-from pixelcore/backend:latest ...

# Frontend - 缓存 node_modules
docker build --cache-from pixelcore/frontend:latest ...
```

3. **并行构建**
```bash
# 同时构建两个镜像
docker build -t pixelcore/backend:latest -f Dockerfile . &
docker build -t pixelcore/frontend:latest -f app/Dockerfile app/ &
wait
```

### 减小镜像大小

当前镜像大小：
- Backend: ~100-150MB (debian-slim + Rust 二进制)
- Frontend: ~20-30MB (nginx-alpine + 静态文件)

优化建议：
1. 使用 `alpine` 基础镜像（Backend 可以考虑 `rust:alpine`）
2. 清理构建缓存和临时文件
3. 使用 `.dockerignore` 排除不必要的文件
4. 压缩静态资源（Frontend）

## 故障排查

### 构建失败

**问题**: 无法拉取基础镜像
```
Error: failed to authorize: failed to fetch anonymous token
```

**解决方案**:
1. 检查网络连接
2. 配置 Docker 镜像加速器
3. 手动拉取基础镜像：
   ```bash
   docker pull rust:1.75-slim
   docker pull debian:bookworm-slim
   docker pull node:20-alpine
   docker pull nginx:alpine
   ```

**问题**: Rust 编译失败
```
Error: could not compile `pixelcore`
```

**解决方案**:
1. 检查 Cargo.toml 依赖
2. 确保所有 crates 都存在
3. 本地测试编译：`cargo build --release`

**问题**: Frontend 构建失败
```
Error: npm run build failed
```

**解决方案**:
1. 检查 package.json 依赖
2. 本地测试构建：`cd app && npm install && npm run build`
3. 检查 TypeScript 错误

### 运行时问题

**问题**: 容器启动后立即退出

**解决方案**:
```bash
# 查看容器日志
docker logs <container-id>

# 检查健康检查
docker inspect <container-id> | grep -A 10 Health
```

**问题**: 健康检查失败

**解决方案**:
1. Backend: 确保 `/health` 端点可访问
2. Frontend: 确保 nginx 配置正确
3. 检查端口映射是否正确

## 相关文件

- `Dockerfile` - Backend Dockerfile
- `app/Dockerfile` - Frontend Dockerfile
- `deployment/scripts/build-images.sh` - 构建脚本
- `deployment/scripts/update-deployment-images.sh` - 更新部署脚本
- `.dockerignore` - Docker 忽略文件

## 下一步

1. ✅ 构建镜像
2. ✅ 测试镜像
3. ⏭️ 推送到镜像仓库
4. ⏭️ 更新 Kubernetes 部署
5. ⏭️ 验证生产环境
