# Docker 部署指南

本文档介绍如何使用 Docker 和 Docker Compose 部署 PixelCore。

---

## 📋 前置要求

- Docker 20.10+
- Docker Compose 2.0+
- 至少 4GB RAM
- 至少 20GB 磁盘空间

---

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/your-org/pixelcore.git
cd pixelcore
```

### 2. 配置环境变量

```bash
cp .env.docker.example .env.docker
```

编辑 `.env.docker` 文件，设置安全的密码：

```env
POSTGRES_PASSWORD=your_secure_postgres_password
REDIS_PASSWORD=your_secure_redis_password
GRAFANA_PASSWORD=your_secure_grafana_password
```

### 3. 启动服务

```bash
# 构建并启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 查看服务状态
docker-compose ps
```

### 4. 访问应用

- **前端**: http://localhost
- **后端 API**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

---

## 🏗️ 架构说明

### 服务组件

| 服务 | 端口 | 说明 |
|------|------|------|
| frontend | 80 | React 前端 (Nginx) |
| backend | 8080 | Rust 后端 API |
| postgres | 5432 | PostgreSQL 数据库 |
| redis | 6379 | Redis 缓存 |
| prometheus | 9090 | Prometheus 监控 |
| grafana | 3000 | Grafana 可视化 |

### 网络拓扑

```
┌─────────────┐
│  Frontend   │ :80
│   (Nginx)   │
└──────┬──────┘
       │
       ├─────────────┐
       │             │
┌──────▼──────┐ ┌───▼────────┐
│   Backend   │ │ Prometheus │
│   (Rust)    │ │            │
└──────┬──────┘ └───┬────────┘
       │            │
   ┌───┴────┬───────┴───┐
   │        │           │
┌──▼───┐ ┌─▼────┐ ┌────▼────┐
│Postgres│Redis│ │ Grafana │
└────────┘└──────┘ └─────────┘
```

---

## 📦 镜像构建

### 构建后端镜像

```bash
docker build -t pixelcore-backend:latest .
```

### 构建前端镜像

```bash
cd app
docker build -t pixelcore-frontend:latest .
```

### 推送到镜像仓库

```bash
# 标记镜像
docker tag pixelcore-backend:latest your-registry/pixelcore-backend:latest
docker tag pixelcore-frontend:latest your-registry/pixelcore-frontend:latest

# 推送镜像
docker push your-registry/pixelcore-backend:latest
docker push your-registry/pixelcore-frontend:latest
```

---

## 🔧 常用命令

### 服务管理

```bash
# 启动服务
docker-compose up -d

# 停止服务
docker-compose stop

# 重启服务
docker-compose restart

# 停止并删除容器
docker-compose down

# 停止并删除容器和卷
docker-compose down -v
```

### 日志查看

```bash
# 查看所有服务日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f backend
docker-compose logs -f frontend

# 查看最近 100 行日志
docker-compose logs --tail=100 backend
```

### 服务扩展

```bash
# 扩展后端服务到 3 个实例
docker-compose up -d --scale backend=3

# 查看服务状态
docker-compose ps
```

### 数据库管理

```bash
# 进入 PostgreSQL 容器
docker-compose exec postgres psql -U pixelcore -d pixelcore

# 备份数据库
docker-compose exec postgres pg_dump -U pixelcore pixelcore > backup.sql

# 恢复数据库
docker-compose exec -T postgres psql -U pixelcore pixelcore < backup.sql
```

### 缓存管理

```bash
# 进入 Redis 容器
docker-compose exec redis redis-cli -a your_redis_password

# 清空缓存
docker-compose exec redis redis-cli -a your_redis_password FLUSHALL
```

---

## 🔍 健康检查

### 检查服务健康状态

```bash
# 查看所有服务健康状态
docker-compose ps

# 检查后端健康
curl http://localhost:8080/health

# 检查前端健康
curl http://localhost/health
```

### 健康检查端点

| 服务 | 端点 | 预期响应 |
|------|------|----------|
| Backend | http://localhost:8080/health | 200 OK |
| Frontend | http://localhost/health | 200 OK |
| Prometheus | http://localhost:9090/-/healthy | 200 OK |
| Grafana | http://localhost:3000/api/health | 200 OK |

---

## 📊 监控和日志

### Prometheus 监控

访问 http://localhost:9090 查看 Prometheus 监控面板。

常用查询：
```promql
# CPU 使用率
rate(process_cpu_seconds_total[5m])

# 内存使用
process_resident_memory_bytes

# HTTP 请求速率
rate(http_requests_total[5m])

# HTTP 请求延迟
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

### Grafana 可视化

1. 访问 http://localhost:3000
2. 使用默认凭据登录: admin/admin
3. 添加 Prometheus 数据源: http://prometheus:9090
4. 导入预配置的仪表板

---

## 🔒 安全配置

### 1. 更改默认密码

编辑 `.env.docker` 文件，设置强密码：

```env
POSTGRES_PASSWORD=<strong-password>
REDIS_PASSWORD=<strong-password>
GRAFANA_PASSWORD=<strong-password>
```

### 2. 启用 HTTPS

使用 Let's Encrypt 或自签名证书：

```bash
# 生成自签名证书
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout ./certs/privkey.pem \
  -out ./certs/fullchain.pem
```

更新 `docker-compose.yml` 中的 Nginx 配置。

### 3. 网络隔离

默认情况下，所有服务都在 `pixelcore-network` 网络中。
不要将数据库端口暴露到主机。

---

## 🐛 故障排查

### 服务无法启动

```bash
# 查看详细日志
docker-compose logs backend

# 检查容器状态
docker-compose ps

# 重新构建镜像
docker-compose build --no-cache
```

### 数据库连接失败

```bash
# 检查 PostgreSQL 是否运行
docker-compose ps postgres

# 检查数据库日志
docker-compose logs postgres

# 测试数据库连接
docker-compose exec postgres psql -U pixelcore -d pixelcore -c "SELECT 1"
```

### 缓存连接失败

```bash
# 检查 Redis 是否运行
docker-compose ps redis

# 检查 Redis 日志
docker-compose logs redis

# 测试 Redis 连接
docker-compose exec redis redis-cli -a your_redis_password PING
```

### 磁盘空间不足

```bash
# 清理未使用的镜像
docker image prune -a

# 清理未使用的卷
docker volume prune

# 清理未使用的网络
docker network prune

# 清理所有未使用的资源
docker system prune -a --volumes
```

---

## 🔄 更新和升级

### 更新应用

```bash
# 拉取最新代码
git pull

# 重新构建镜像
docker-compose build

# 重启服务
docker-compose up -d
```

### 滚动更新

```bash
# 逐个重启服务，避免停机
docker-compose up -d --no-deps --build backend
docker-compose up -d --no-deps --build frontend
```

---

## 📈 性能优化

### 1. 资源限制

在 `docker-compose.yml` 中添加资源限制：

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

### 2. 缓存优化

- 启用 Redis 持久化
- 配置合理的缓存过期时间
- 使用缓存预热

### 3. 数据库优化

- 配置连接池
- 启用查询缓存
- 定期执行 VACUUM

---

## 🌐 生产部署建议

### 1. 使用外部数据库

生产环境建议使用托管数据库服务（如 AWS RDS、Google Cloud SQL）。

### 2. 使用负载均衡

在多个后端实例前添加负载均衡器（如 Nginx、HAProxy）。

### 3. 启用自动备份

配置定期数据库备份和灾难恢复计划。

### 4. 监控和告警

配置 Prometheus Alertmanager 发送告警通知。

### 5. 日志聚合

使用 ELK Stack 或 Loki 进行日志聚合和分析。

---

## 📚 参考资料

- [Docker 官方文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)

---

## 🆘 获取帮助

如果遇到问题，请：

1. 查看日志: `docker-compose logs -f`
2. 检查服务状态: `docker-compose ps`
3. 查看本文档的故障排查部分
4. 提交 Issue: https://github.com/your-org/pixelcore/issues

---

**部署愉快！** 🚀
