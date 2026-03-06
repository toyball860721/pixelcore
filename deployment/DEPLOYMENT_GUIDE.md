# PixelCore 生产环境部署指南

## 概述

本指南提供完整的生产环境部署步骤，包括基础设施、应用服务、监控系统和备份系统的部署。

## 前置条件

### 必需工具
- kubectl (v1.24+)
- helm (v3.0+)
- velero CLI (v1.12+)
- git

### 集群要求
- Kubernetes 集群 (v1.24+)
- 至少 3 个工作节点
- 每个节点至少 4 CPU, 8GB RAM
- 支持 PersistentVolume
- 支持 LoadBalancer (或 Ingress Controller)

### 权限要求
- 集群管理员权限
- 创建命名空间权限
- 部署资源权限

## 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/toyball860721/pixelcore.git
cd pixelcore
```

### 2. 配置环境变量

```bash
# 复制配置模板
cp deployment/configs/production.env.example deployment/configs/production.env

# 编辑配置
vim deployment/configs/production.env
```

必须配置的变量：
- `POSTGRES_PASSWORD` - PostgreSQL密码
- `REDIS_PASSWORD` - Redis密码
- `SLACK_WEBHOOK_URL` - Slack告警webhook
- `BACKUP_BUCKET` - 备份存储桶名称
- `DOMAIN` - 生产域名

### 3. 创建Secret

```bash
# 创建数据库密码
kubectl create secret generic pixelcore-secrets \
  --from-literal=POSTGRES_PASSWORD=<your-password> \
  --from-literal=REDIS_PASSWORD=<your-password> \
  -n pixelcore

# 创建AlertManager密码
kubectl create secret generic alertmanager-secrets \
  --from-literal=slack-webhook-url=<your-webhook> \
  --from-literal=smtp-password=<your-smtp-password> \
  -n monitoring
```

### 4. 安装Velero备份系统

```bash
cd deployment/scripts
chmod +x install-velero.sh
./install-velero.sh
```

### 5. 部署生产环境

```bash
chmod +x deploy-production.sh
./deploy-production.sh
```

### 6. 验证部署

```bash
cd ../validation
chmod +x health-check.sh
./health-check.sh
```

## 详细部署步骤

### 步骤 1: 准备集群

#### 1.1 验证集群连接

```bash
kubectl cluster-info
kubectl get nodes
```

#### 1.2 创建命名空间

```bash
kubectl create namespace pixelcore
kubectl create namespace monitoring
kubectl create namespace velero
```

#### 1.3 标记命名空间

```bash
kubectl label namespace pixelcore environment=production
kubectl label namespace monitoring environment=production
```

### 步骤 2: 部署基础设施

#### 2.1 部署PostgreSQL HA

```bash
kubectl apply -f k8s/base/postgres-ha.yaml
kubectl apply -f k8s/base/postgres-pdb.yaml

# 等待就绪
kubectl wait --for=condition=Ready pods -l app=postgres-ha -n pixelcore --timeout=600s

# 验证复制
kubectl exec -it postgres-ha-0 -n pixelcore -- \
  psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"
```

#### 2.2 部署Redis HA

```bash
kubectl apply -f k8s/base/redis-ha.yaml
kubectl apply -f k8s/base/redis-pdb.yaml

# 等待就绪
kubectl wait --for=condition=Ready pods -l app=redis-ha -n pixelcore --timeout=600s

# 验证Sentinel
kubectl exec -it redis-sentinel-0 -n pixelcore -- \
  redis-cli -p 26379 SENTINEL masters
```

### 步骤 3: 部署监控系统

#### 3.1 部署Prometheus告警规则

```bash
kubectl apply -f monitoring/alerts/reliability-rules.yaml
kubectl apply -f monitoring/alerts/availability-rules.yaml
kubectl apply -f monitoring/alerts/performance-rules.yaml
```

#### 3.2 部署AlertManager

```bash
kubectl apply -f monitoring/alertmanager-config.yaml

# 等待就绪
kubectl wait --for=condition=Ready pods -l app=alertmanager -n monitoring --timeout=300s
```

#### 3.3 更新Prometheus配置

```bash
kubectl create configmap prometheus-config \
  --from-file=monitoring/prometheus.yml \
  -n monitoring \
  --dry-run=client -o yaml | kubectl apply -f -

# 重启Prometheus
kubectl rollout restart statefulset prometheus -n monitoring
```

### 步骤 4: 部署服务网格

#### 4.1 部署熔断器

```bash
kubectl apply -f k8s/service-mesh/destination-rules/circuit-breakers.yaml
```

#### 4.2 部署重试策略

```bash
kubectl apply -f k8s/service-mesh/virtual-services/retry-policies.yaml
```

### 步骤 5: 部署备份系统

#### 5.1 部署备份计划

```bash
kubectl apply -f reliability/velero-schedules.yaml
kubectl apply -f reliability/backup-storage-class.yaml
```

#### 5.2 验证备份

```bash
# 触发手动备份
velero backup create manual-test --wait

# 验证备份
./reliability/verify-backup.sh manual-test
```

### 步骤 6: 部署应用服务

#### 6.1 部署配置

```bash
kubectl apply -f k8s/base/configmap.yaml
kubectl apply -f k8s/base/secret.yaml
```

#### 6.2 部署后端服务

```bash
kubectl apply -f k8s/base/backend.yaml

# 等待就绪
kubectl wait --for=condition=Ready pods -l app=backend -n pixelcore --timeout=300s
```

#### 6.3 部署前端服务

```bash
kubectl apply -f k8s/base/frontend.yaml

# 等待就绪
kubectl wait --for=condition=Ready pods -l app=frontend -n pixelcore --timeout=300s
```

#### 6.4 部署HPA和PDB

```bash
kubectl apply -f k8s/base/hpa.yaml
kubectl apply -f k8s/base/pdb.yaml
```

#### 6.5 部署Ingress

```bash
kubectl apply -f k8s/base/ingress.yaml
```

### 步骤 7: 验证部署

#### 7.1 运行健康检查

```bash
./deployment/validation/health-check.sh
```

#### 7.2 运行可靠性测试

```bash
./reliability/tests/reliability-test-suite.sh
```

#### 7.3 检查监控

```bash
# Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# Grafana
kubectl port-forward -n monitoring svc/grafana 3000:3000

# AlertManager
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
```

## 部署后配置

### 1. 配置域名和SSL

```bash
# 获取Ingress IP
kubectl get ingress -n pixelcore

# 配置DNS记录
# A记录: pixelcore.example.com -> <INGRESS_IP>

# 配置SSL证书（使用cert-manager）
kubectl apply -f k8s/base/certificate.yaml
```

### 2. 配置告警通知

```bash
# 测试Slack通知
kubectl exec -it -n monitoring alertmanager-0 -- \
  amtool alert add test severity=warning

# 验证收到通知
```

### 3. 配置备份验证

```bash
# 设置定时任务验证备份
kubectl create cronjob backup-verification \
  --image=bitnami/kubectl \
  --schedule="0 3 * * *" \
  --restart=Never \
  -- /bin/bash -c "./reliability/verify-backup.sh"
```

## 监控和维护

### 日常检查

```bash
# 每日健康检查
./deployment/validation/health-check.sh

# 检查备份状态
velero backup get

# 检查告警
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
```

### 性能监控

```bash
# 查看资源使用
kubectl top nodes
kubectl top pods -n pixelcore

# 查看HPA状态
kubectl get hpa -n pixelcore
```

### 日志查看

```bash
# 查看应用日志
kubectl logs -f -l app=backend -n pixelcore

# 查看数据库日志
kubectl logs -f postgres-ha-0 -n pixelcore

# 查看告警日志
kubectl logs -f -l app=alertmanager -n monitoring
```

## 故障排查

### Pod无法启动

```bash
# 查看Pod状态
kubectl describe pod <pod-name> -n pixelcore

# 查看事件
kubectl get events -n pixelcore --sort-by='.lastTimestamp'

# 查看日志
kubectl logs <pod-name> -n pixelcore
```

### 数据库连接失败

```bash
# 检查PostgreSQL状态
kubectl exec -it postgres-ha-0 -n pixelcore -- pg_isready

# 检查复制状态
kubectl exec -it postgres-ha-0 -n pixelcore -- \
  psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"

# 检查连接
kubectl exec -it <backend-pod> -n pixelcore -- \
  nc -zv postgres-ha-service 5432
```

### 告警未触发

```bash
# 检查Prometheus规则
kubectl exec -it -n monitoring prometheus-0 -- \
  promtool check rules /etc/prometheus/alerts/*.yaml

# 检查AlertManager配置
kubectl exec -it -n monitoring alertmanager-0 -- \
  amtool config show

# 手动触发测试告警
kubectl scale deployment backend -n pixelcore --replicas=0
```

## 回滚

### 回滚应用

```bash
# 使用回滚脚本
./deployment/scripts/rollback.sh

# 或手动回滚
kubectl rollout undo deployment/backend -n pixelcore
```

### 从备份恢复

```bash
# 列出备份
velero backup get

# 恢复
velero restore create --from-backup <backup-name> --wait
```

## 安全注意事项

1. **密钥管理**
   - 不要在代码中硬编码密钥
   - 使用Kubernetes Secret或Vault
   - 定期轮换密钥

2. **网络安全**
   - 启用网络策略
   - 使用mTLS
   - 限制入站流量

3. **访问控制**
   - 使用RBAC
   - 最小权限原则
   - 定期审计权限

4. **监控和审计**
   - 启用审计日志
   - 监控异常活动
   - 定期安全扫描

## 性能优化

1. **资源调优**
   - 根据实际负载调整资源限制
   - 优化HPA配置
   - 使用节点亲和性

2. **缓存优化**
   - 配置Redis缓存
   - 使用CDN
   - 启用HTTP缓存

3. **数据库优化**
   - 优化查询
   - 配置连接池
   - 使用读写分离

## 支持和帮助

- **文档**: docs/RELIABILITY.md
- **Runbooks**: reliability/runbooks/
- **快速参考**: RELIABILITY_QUICK_REFERENCE.md
- **问题反馈**: GitHub Issues

## 附录

### A. 常用命令

```bash
# 查看所有资源
kubectl get all -n pixelcore

# 查看Pod详情
kubectl describe pod <pod-name> -n pixelcore

# 进入Pod
kubectl exec -it <pod-name> -n pixelcore -- /bin/bash

# 查看日志
kubectl logs -f <pod-name> -n pixelcore

# 端口转发
kubectl port-forward <pod-name> 8080:8080 -n pixelcore
```

### B. 配置文件位置

- 应用配置: `k8s/base/`
- 监控配置: `monitoring/`
- 可靠性配置: `reliability/`
- 服务网格: `k8s/service-mesh/`
- 部署脚本: `deployment/scripts/`

### C. 联系方式

- 技术支持: support@pixelcore.com
- 紧急联系: oncall@pixelcore.com
- Slack: #pixelcore-ops
