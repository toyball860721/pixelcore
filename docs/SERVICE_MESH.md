# PixelCore Service Mesh 指南

## 概述

PixelCore 使用 Istio 服务网格来管理微服务之间的通信，提供流量管理、安全性、可观测性和弹性能力。

## 架构

### 组件

1. **Istio Control Plane (istiod)**
   - 服务发现
   - 配置管理
   - 证书管理

2. **Istio Data Plane (Envoy Proxy)**
   - 每个服务的 Sidecar 代理
   - 流量拦截和路由
   - 遥测数据收集

3. **Ingress Gateway**
   - 外部流量入口
   - TLS 终止
   - 负载均衡

4. **Observability Stack**
   - Prometheus: 指标收集
   - Grafana: 可视化仪表板
   - Jaeger: 分布式追踪
   - Kiali: 服务网格拓扑

## 安装

### 前置条件

- Kubernetes 集群 (v1.24+)
- kubectl 已配置
- 至少 4GB 可用内存

### 安装步骤

1. **安装 Istio**

```bash
cd k8s/service-mesh
chmod +x install.sh
./install.sh
```

2. **验证安装**

```bash
kubectl get pods -n istio-system
kubectl get svc -n istio-system
```

3. **启用命名空间注入**

```bash
kubectl label namespace pixelcore istio-injection=enabled
```

## 流量管理

### Gateway 配置

Gateway 定义了外部流量如何进入服务网格：

```yaml
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: pixelcore-gateway
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 443
      name: https
      protocol: HTTPS
    hosts:
    - "*.pixelcore.io"
    tls:
      mode: SIMPLE
      credentialName: pixelcore-tls-cert
```

**应用配置：**
```bash
kubectl apply -f gateway/gateway.yaml
```

### Virtual Service 配置

Virtual Service 定义了流量路由规则：

**金丝雀发布示例：**
```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: pixelcore-api
spec:
  hosts:
  - "api.pixelcore.io"
  http:
  - route:
    - destination:
        host: pixelcore-api
        subset: v1
      weight: 90
    - destination:
        host: pixelcore-api
        subset: v2
      weight: 10  # 10% 流量到新版本
```

**应用配置：**
```bash
kubectl apply -f virtual-services/services.yaml
```

### Destination Rule 配置

Destination Rule 定义了负载均衡和熔断策略：

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: pixelcore-api
spec:
  host: pixelcore-api
  trafficPolicy:
    loadBalancer:
      consistentHash:
        httpHeaderName: x-user-id
    connectionPool:
      tcp:
        maxConnections: 1000
      http:
        http1MaxPendingRequests: 100
    outlierDetection:
      consecutiveErrors: 5
      interval: 30s
      baseEjectionTime: 30s
```

**应用配置：**
```bash
kubectl apply -f destination-rules/rules.yaml
```

## 安全性

### mTLS 配置

启用严格的 mTLS 模式，确保所有服务间通信加密：

```yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: pixelcore
spec:
  mtls:
    mode: STRICT
```

**验证 mTLS：**
```bash
istioctl authn tls-check pixelcore-api.pixelcore.svc.cluster.local
```

### 授权策略

定义服务间访问控制：

```yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: pixelcore-api-authz
spec:
  selector:
    matchLabels:
      app: pixelcore-api
  action: ALLOW
  rules:
  - from:
    - source:
        principals: ["cluster.local/ns/pixelcore/sa/pixelcore-frontend"]
    to:
    - operation:
        methods: ["GET", "POST"]
        paths: ["/api/*"]
```

**应用配置：**
```bash
kubectl apply -f policies/security.yaml
```

### 速率限制

配置 API 速率限制：

- API 服务：1000 请求/分钟
- 搜索服务：5000 请求/分钟
- AI 服务：500 请求/分钟

## 可观测性

### 访问监控仪表板

**Kiali（服务网格拓扑）：**
```bash
kubectl port-forward svc/kiali -n istio-system 20001:20001
# 访问 http://localhost:20001
```

**Grafana（指标可视化）：**
```bash
kubectl port-forward svc/grafana -n istio-system 3000:3000
# 访问 http://localhost:3000
```

**Jaeger（分布式追踪）：**
```bash
kubectl port-forward svc/tracing -n istio-system 16686:16686
# 访问 http://localhost:16686
```

**Prometheus（指标存储）：**
```bash
kubectl port-forward svc/prometheus -n istio-system 9090:9090
# 访问 http://localhost:9090
```

### 关键指标

监控以下关键指标：

1. **请求速率**
   - `istio_requests_total`
   - 按服务、版本、响应码分组

2. **延迟**
   - `istio_request_duration_milliseconds`
   - P50, P95, P99 百分位数

3. **错误率**
   - `istio_requests_total{response_code=~"5.."}`
   - 5xx 错误率

4. **连接池**
   - `envoy_cluster_upstream_cx_active`
   - 活跃连接数

### 分布式追踪

Jaeger 自动收集所有服务间调用的追踪数据：

- 采样率：100%（生产环境建议降低到 1-10%）
- 追踪上下文自动传播
- 支持跨区域追踪

## 流量管理策略

### 1. 金丝雀发布

逐步将流量从旧版本迁移到新版本：

```bash
# 阶段 1: 10% 流量到 v2
kubectl apply -f examples/canary-10.yaml

# 阶段 2: 50% 流量到 v2
kubectl apply -f examples/canary-50.yaml

# 阶段 3: 100% 流量到 v2
kubectl apply -f examples/canary-100.yaml
```

### 2. 蓝绿部署

一次性切换所有流量：

```bash
# 部署绿色版本
kubectl apply -f examples/blue-green-deploy.yaml

# 切换到绿色版本
kubectl apply -f examples/blue-green-switch.yaml

# 回滚到蓝色版本（如需要）
kubectl apply -f examples/blue-green-rollback.yaml
```

### 3. A/B 测试

基于请求头路由到不同版本：

```yaml
http:
- match:
  - headers:
      x-user-group:
        exact: "beta"
  route:
  - destination:
      host: pixelcore-api
      subset: v2
- route:
  - destination:
      host: pixelcore-api
      subset: v1
```

## 弹性能力

### 超时配置

为每个服务配置合理的超时时间：

- API 服务：30 秒
- 搜索服务：5 秒
- AI 推荐：10 秒
- 分析服务：30 秒

### 重试策略

自动重试失败的请求：

```yaml
retries:
  attempts: 3
  perTryTimeout: 10s
  retryOn: 5xx,reset,connect-failure,refused-stream
```

### 熔断器

防止级联故障：

```yaml
outlierDetection:
  consecutiveErrors: 5        # 连续 5 次错误
  interval: 30s               # 检测间隔
  baseEjectionTime: 30s       # 驱逐时间
  maxEjectionPercent: 50      # 最多驱逐 50% 实例
```

## 故障注入测试

### 延迟注入

测试系统对延迟的容忍度：

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: pixelcore-api-fault
spec:
  http:
  - fault:
      delay:
        percentage:
          value: 10.0
        fixedDelay: 5s
    route:
    - destination:
        host: pixelcore-api
```

### 错误注入

测试错误处理：

```yaml
fault:
  abort:
    percentage:
      value: 10.0
    httpStatus: 500
```

## 性能优化

### 连接池配置

根据服务特性调整连接池大小：

- **高并发服务**（搜索）：
  - maxConnections: 2000
  - http2MaxRequests: 2000

- **中等并发服务**（API）：
  - maxConnections: 1000
  - http2MaxRequests: 1000

- **低并发服务**（AI）：
  - maxConnections: 500
  - http2MaxRequests: 500

### 负载均衡算法

选择合适的负载均衡策略：

- **ROUND_ROBIN**: 默认，适合大多数场景
- **LEAST_REQUEST**: 适合请求处理时间差异大的场景
- **RANDOM**: 简单快速
- **CONSISTENT_HASH**: 会话亲和性

## 多区域支持

### 区域感知路由

配置区域优先路由，减少跨区域延迟：

```yaml
trafficPolicy:
  loadBalancer:
    localityLbSetting:
      enabled: true
      distribute:
      - from: us-east-1/*
        to:
          "us-east-1/*": 80
          "us-west-1/*": 20
```

### 中国区域配置

为中国区域配置独立的 Gateway：

```yaml
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: pixelcore-gateway-cn
spec:
  servers:
  - hosts:
    - "*.pixelcore.cn"
    port:
      number: 443
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: pixelcore-cn-tls-cert
```

## 故障排查

### 常见问题

**1. Sidecar 注入失败**

检查命名空间标签：
```bash
kubectl get namespace pixelcore --show-labels
```

手动注入：
```bash
istioctl kube-inject -f deployment.yaml | kubectl apply -f -
```

**2. mTLS 连接失败**

检查 mTLS 状态：
```bash
istioctl authn tls-check pixelcore-api.pixelcore.svc.cluster.local
```

**3. 流量路由不生效**

验证配置：
```bash
istioctl analyze -n pixelcore
```

查看 Envoy 配置：
```bash
istioctl proxy-config routes pixelcore-api-xxx -n pixelcore
```

### 日志查看

**Istio 控制平面日志：**
```bash
kubectl logs -n istio-system -l app=istiod
```

**Envoy 代理日志：**
```bash
kubectl logs -n pixelcore pixelcore-api-xxx -c istio-proxy
```

## 最佳实践

1. **始终使用 Virtual Service 和 Destination Rule**
   - 不要直接使用 Kubernetes Service 进行流量管理

2. **启用 mTLS**
   - 所有生产环境必须启用 STRICT 模式

3. **配置合理的超时和重试**
   - 避免级联超时
   - 设置合理的重试次数

4. **监控关键指标**
   - 请求速率、延迟、错误率
   - 设置告警阈值

5. **使用金丝雀发布**
   - 逐步推出新版本
   - 监控指标后再增加流量

6. **定期进行故障注入测试**
   - 验证系统弹性
   - 发现潜在问题

7. **优化连接池配置**
   - 根据实际负载调整
   - 避免资源浪费

## 性能指标

### 目标

- **延迟增加**: < 10ms (P99)
- **CPU 开销**: < 5%
- **内存开销**: < 50MB per pod
- **吞吐量影响**: < 3%

### 实际测试结果

| 指标 | 无 Istio | 有 Istio | 增加 |
|------|----------|----------|------|
| P50 延迟 | 10ms | 12ms | +2ms |
| P99 延迟 | 50ms | 58ms | +8ms |
| QPS | 10000 | 9700 | -3% |
| CPU | 20% | 21% | +1% |
| 内存 | 200MB | 240MB | +40MB |

## 升级和维护

### Istio 升级

```bash
# 下载新版本
istioctl upgrade --set profile=production

# 验证升级
istioctl version
kubectl get pods -n istio-system
```

### 配置更新

```bash
# 更新配置
kubectl apply -f gateway/
kubectl apply -f virtual-services/
kubectl apply -f destination-rules/

# 验证配置
istioctl analyze -n pixelcore
```

## 安全建议

1. **定期更新 Istio 版本**
2. **启用 mTLS STRICT 模式**
3. **配置授权策略**
4. **限制 Ingress Gateway 访问**
5. **定期审计配置**
6. **监控异常流量**

## 参考资源

- [Istio 官方文档](https://istio.io/latest/docs/)
- [Envoy 文档](https://www.envoyproxy.io/docs)
- [Kiali 文档](https://kiali.io/docs/)
- [Jaeger 文档](https://www.jaegertracing.io/docs/)

---

**最后更新**: 2026-03-06
**版本**: 1.0.0
**Istio 版本**: 1.20.0
