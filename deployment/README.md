# PixelCore 部署工具

生产环境部署脚本和工具集。

## 目录结构

```
deployment/
├── scripts/              # 部署脚本
│   ├── deploy-production.sh    # 主部署脚本
│   ├── install-velero.sh       # Velero安装
│   └── rollback.sh             # 回滚脚本
├── configs/              # 配置文件
│   └── production.env          # 生产环境配置
├── validation/           # 验证脚本
│   └── health-check.sh         # 健康检查
├── DEPLOYMENT_GUIDE.md   # 详细部署指南
└── README.md            # 本文件
```

## 快速开始

### 1. 准备环境

```bash
# 确保kubectl已配置
kubectl cluster-info

# 配置环境变量
cp configs/production.env.example configs/production.env
vim configs/production.env
```

### 2. 创建Secret

```bash
kubectl create secret generic pixelcore-secrets \
  --from-literal=POSTGRES_PASSWORD=<password> \
  --from-literal=REDIS_PASSWORD=<password> \
  -n pixelcore
```

### 3. 安装Velero

```bash
cd scripts
chmod +x install-velero.sh
./install-velero.sh
```

### 4. 部署生产环境

```bash
chmod +x deploy-production.sh
./deploy-production.sh
```

### 5. 验证部署

```bash
cd ../validation
chmod +x health-check.sh
./health-check.sh
```

## 脚本说明

### deploy-production.sh

主部署脚本，执行完整的生产环境部署。

**功能**:
- 检查前置条件
- 创建命名空间
- 部署基础设施（PostgreSQL HA, Redis HA）
- 部署监控系统（Prometheus, AlertManager）
- 部署服务网格（Istio配置）
- 部署备份系统（Velero）
- 部署应用服务
- 验证部署

**使用方法**:
```bash
# 正常部署
./deploy-production.sh

# Dry Run模式
DRY_RUN=true ./deploy-production.sh

# 自定义命名空间
NAMESPACE=my-namespace ./deploy-production.sh
```

### install-velero.sh

安装Velero备份系统。

**支持的云提供商**:
- AWS
- GCP
- Azure

**使用方法**:
```bash
# AWS
CLOUD_PROVIDER=aws BUCKET_NAME=my-bucket ./install-velero.sh

# GCP
CLOUD_PROVIDER=gcp BUCKET_NAME=my-bucket ./install-velero.sh

# Azure
CLOUD_PROVIDER=azure BUCKET_NAME=my-bucket ./install-velero.sh
```

### rollback.sh

回滚脚本，支持多种回滚方式。

**功能**:
- 回滚所有服务
- 回滚特定服务
- 从Velero备份恢复

**使用方法**:
```bash
./rollback.sh
```

### health-check.sh

健康检查脚本，验证系统状态。

**检查项**:
- 命名空间
- Pod状态
- 数据库健康
- Redis健康
- 服务状态
- HPA配置
- PDB配置
- 监控系统
- 备份系统
- 资源使用

**使用方法**:
```bash
./health-check.sh

# 自定义命名空间
NAMESPACE=my-namespace ./health-check.sh
```

## 配置说明

### production.env

生产环境配置文件。

**必须配置的变量**:
- `POSTGRES_PASSWORD` - PostgreSQL密码
- `REDIS_PASSWORD` - Redis密码
- `SLACK_WEBHOOK_URL` - Slack告警webhook
- `BACKUP_BUCKET` - 备份存储桶
- `DOMAIN` - 生产域名

**可选配置**:
- 资源限制
- HPA配置
- 功能开关

## 部署流程

```
1. 前置检查
   ├── kubectl连接
   ├── helm安装
   └── 权限验证

2. 基础设施
   ├── PostgreSQL HA (3副本)
   ├── Redis HA (3副本)
   └── Redis Sentinel (3副本)

3. 监控系统
   ├── Prometheus告警规则
   ├── AlertManager
   └── Grafana仪表板

4. 服务网格
   ├── 熔断器
   └── 重试策略

5. 备份系统
   ├── Velero安装
   └── 备份计划

6. 应用服务
   ├── 后端服务
   ├── 前端服务
   ├── HPA
   ├── PDB
   └── Ingress

7. 验证
   ├── 健康检查
   └── 可靠性测试
```

## 故障排查

### 部署失败

```bash
# 查看部署日志
kubectl get events -n pixelcore --sort-by='.lastTimestamp'

# 查看Pod状态
kubectl get pods -n pixelcore

# 查看Pod详情
kubectl describe pod <pod-name> -n pixelcore
```

### 健康检查失败

```bash
# 运行详细检查
./validation/health-check.sh

# 查看具体服务
kubectl get all -n pixelcore
```

### 回滚

```bash
# 使用回滚脚本
./scripts/rollback.sh

# 或从备份恢复
velero restore create --from-backup <backup-name>
```

## 最佳实践

1. **部署前**
   - 在测试环境验证
   - 备份当前状态
   - 准备回滚计划

2. **部署中**
   - 监控部署进度
   - 检查日志
   - 验证每个步骤

3. **部署后**
   - 运行健康检查
   - 验证监控告警
   - 测试关键功能

## 安全建议

1. **密钥管理**
   - 使用Kubernetes Secret
   - 不要提交密钥到Git
   - 定期轮换密钥

2. **访问控制**
   - 使用RBAC
   - 最小权限原则
   - 定期审计

3. **网络安全**
   - 启用网络策略
   - 使用mTLS
   - 限制入站流量

## 相关文档

- [详细部署指南](DEPLOYMENT_GUIDE.md)
- [可靠性文档](../docs/RELIABILITY.md)
- [快速参考](../RELIABILITY_QUICK_REFERENCE.md)
- [灾难恢复](../reliability/runbooks/disaster-recovery.md)

## 支持

- GitHub Issues: https://github.com/toyball860721/pixelcore/issues
- 文档: docs/
- Slack: #pixelcore-ops

## 许可证

Copyright © 2026 PixelCore
