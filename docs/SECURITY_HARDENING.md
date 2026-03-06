# PixelCore 安全加固指南

## 概述

本文档提供 PixelCore 系统的安全加固指南，包括漏洞扫描、安全策略和最佳实践。

## 安全目标

### 漏洞管理
- 0 高危漏洞
- 0 中危漏洞
- 低危漏洞 < 10 个
- 定期安全扫描

### 访问控制
- 最小权限原则
- 强制 MFA 认证
- 会话超时: 1 小时
- IP 白名单

### 数据保护
- 所有敏感数据加密
- TLS 1.3 强制启用
- 密钥定期轮换（90 天）
- 完整审计日志

### 合规性
- GDPR 合规
- SOC 2 合规
- ISO 27001 合规
- OWASP Top 10 防护

## 安全扫描

### 1. 容器镜像扫描

**使用 Trivy 扫描**:

```bash
# 扫描单个镜像
trivy image pixelcore/api:latest

# 扫描所有镜像
for image in api search ai analytics; do
  trivy image pixelcore/$image:latest
done

# 生成 JSON 报告
trivy image --format json --output report.json pixelcore/api:latest
```

**配置文件**: `security/trivy.yaml`

```yaml
severity:
  - CRITICAL
  - HIGH
  - MEDIUM

exit-code: 1  # 发现漏洞时失败
ignore-unfixed: false
```

### 2. 依赖扫描

**Rust 依赖扫描**:

```bash
# 安装 cargo-audit
cargo install cargo-audit

# 扫描 Rust 依赖
cargo audit

# 修复已知漏洞
cargo audit fix
```

**npm 依赖扫描**:

```bash
# 扫描 npm 依赖
cd app
npm audit

# 自动修复
npm audit fix

# 强制修复（可能破坏兼容性）
npm audit fix --force
```

### 3. 代码安全扫描

**使用 cargo-clippy**:

```bash
# 运行 clippy 检查
cargo clippy -- -D warnings

# 检查所有 workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**使用 ESLint**:

```bash
# 扫描 TypeScript/React 代码
cd app
npm run lint

# 自动修复
npm run lint -- --fix
```

### 4. 密钥扫描

**检查暴露的密钥**:

```bash
# 使用 git-secrets
git secrets --scan

# 使用 trufflehog
trufflehog git file://. --json

# 使用自定义脚本
./security/security-scan.sh
```

### 5. Web 应用扫描

**使用 OWASP ZAP**:

```bash
# 基线扫描
docker run -t owasp/zap2docker-stable zap-baseline.py \
  -t https://api.pixelcore.io \
  -r zap-report.html

# 完整扫描
docker run -t owasp/zap2docker-stable zap-full-scan.py \
  -t https://api.pixelcore.io \
  -r zap-full-report.html
```

## 安全加固

### 1. 容器安全

#### 使用非 root 用户

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# 创建非 root 用户
RUN useradd -m -u 1000 pixelcore
USER pixelcore

COPY --from=builder /app/target/release/pixelcore /usr/local/bin/
CMD ["pixelcore"]
```

#### 只读文件系统

```yaml
# Kubernetes Deployment
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: api
        securityContext:
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          capabilities:
            drop:
              - ALL
```

#### 资源限制

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

### 2. 网络安全

#### Network Policy

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: pixelcore-network-policy
spec:
  podSelector:
    matchLabels:
      app: pixelcore
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: istio-system
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

#### TLS 配置

```yaml
# Istio Gateway TLS
apiVersion: networking.istio.io/v1beta1
kind: Gateway
spec:
  servers:
  - port:
      number: 443
      protocol: HTTPS
    tls:
      mode: SIMPLE
      minProtocolVersion: TLSV1_3
      cipherSuites:
      - TLS_AES_256_GCM_SHA384
      - TLS_CHACHA20_POLY1305_SHA256
```

### 3. 密钥管理

#### 使用 Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: pixelcore-secrets
type: Opaque
stringData:
  database-url: "postgresql://user:pass@localhost/db"
  redis-url: "redis://localhost:6379"
  api-key: "your-api-key"
```

#### 使用 Sealed Secrets

```bash
# 安装 Sealed Secrets
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.18.0/controller.yaml

# 创建加密的 Secret
kubeseal --format yaml < secret.yaml > sealed-secret.yaml

# 应用加密的 Secret
kubectl apply -f sealed-secret.yaml
```

#### 使用 HashiCorp Vault

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: pixelcore-vault
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: vault-agent-config
data:
  vault-agent-config.hcl: |
    vault {
      address = "https://vault.pixelcore.io"
    }

    auto_auth {
      method "kubernetes" {
        mount_path = "auth/kubernetes"
        config = {
          role = "pixelcore-app"
        }
      }
    }

    template {
      source      = "/vault/secrets/database.tmpl"
      destination = "/vault/secrets/database-url"
    }
```

### 4. 访问控制

#### RBAC 配置

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: pixelcore-app
  namespace: pixelcore
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list"]
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: pixelcore-app-binding
  namespace: pixelcore
subjects:
- kind: ServiceAccount
  name: pixelcore-app
roleRef:
  kind: Role
  name: pixelcore-app
  apiGroup: rbac.authorization.k8s.io
```

#### Pod Security Policy

```yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: pixelcore-restricted
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
  readOnlyRootFilesystem: true
```

### 5. 审计日志

#### 启用 Kubernetes 审计

```yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: Metadata
  resources:
  - group: ""
    resources: ["secrets", "configmaps"]
- level: RequestResponse
  resources:
  - group: ""
    resources: ["pods"]
  verbs: ["create", "delete", "patch"]
```

#### 应用层审计

```rust
// Rust 审计日志
use tracing::{info, warn};

#[tracing::instrument]
async fn authenticate_user(username: &str) -> Result<User> {
    info!(
        username = username,
        event = "authentication_attempt",
        "User authentication attempt"
    );

    match verify_credentials(username).await {
        Ok(user) => {
            info!(
                username = username,
                user_id = user.id,
                event = "authentication_success",
                "User authenticated successfully"
            );
            Ok(user)
        }
        Err(e) => {
            warn!(
                username = username,
                error = %e,
                event = "authentication_failure",
                "User authentication failed"
            );
            Err(e)
        }
    }
}
```

## 安全策略

### 1. 密码策略

```yaml
password-policy:
  minLength: 12
  requireUppercase: true
  requireLowercase: true
  requireNumbers: true
  requireSpecialChars: true
  maxAge: 90  # days
  historyCount: 5
  lockoutThreshold: 5
  lockoutDuration: 30  # minutes
```

### 2. 会话管理

```yaml
session-policy:
  timeout: 3600  # 1 hour
  maxSessions: 3
  requireMFA: true
  ipWhitelist:
    - 10.0.0.0/8
    - 172.16.0.0/12
```

### 3. 加密策略

```yaml
encryption-policy:
  algorithm: AES-256-GCM
  keyRotation: 90  # days
  tlsMinVersion: "1.3"
  cipherSuites:
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256
```

## 安全监控

### 1. 运行时安全监控

**使用 Falco**:

```yaml
# Falco 规则
- rule: Unauthorized Process
  desc: Detect unauthorized process execution
  condition: >
    spawned_process and
    container and
    not proc.name in (allowed_processes)
  output: >
    Unauthorized process started
    (user=%user.name command=%proc.cmdline container=%container.name)
  priority: WARNING
```

### 2. 安全告警

```yaml
# Prometheus 告警规则
groups:
- name: security
  rules:
  - alert: HighFailedLoginRate
    expr: rate(auth_failed_total[5m]) > 10
    for: 5m
    annotations:
      summary: "High failed login rate detected"

  - alert: UnauthorizedAccess
    expr: rate(http_requests_total{status="403"}[5m]) > 5
    for: 5m
    annotations:
      summary: "High rate of unauthorized access attempts"

  - alert: SuspiciousActivity
    expr: rate(security_events_total{severity="high"}[5m]) > 1
    for: 1m
    annotations:
      summary: "Suspicious security activity detected"
```

## 合规性

### 1. GDPR 合规

- ✅ 数据加密（传输和静态）
- ✅ 访问控制和审计
- ✅ 数据删除能力
- ✅ 数据导出能力
- ✅ 隐私政策

### 2. SOC 2 合规

- ✅ 访问控制
- ✅ 变更管理
- ✅ 风险评估
- ✅ 事件响应
- ✅ 监控和日志

### 3. ISO 27001 合规

- ✅ 信息安全政策
- ✅ 资产管理
- ✅ 访问控制
- ✅ 加密
- ✅ 物理安全

## 安全检查清单

### 部署前检查

- [ ] 所有容器镜像已扫描
- [ ] 无高危或中危漏洞
- [ ] 所有依赖已更新
- [ ] 密钥已加密存储
- [ ] TLS 1.3 已启用
- [ ] Network Policy 已配置
- [ ] RBAC 已配置
- [ ] 审计日志已启用
- [ ] 监控告警已配置
- [ ] 安全文档已更新

### 定期检查（每周）

- [ ] 运行安全扫描
- [ ] 检查审计日志
- [ ] 审查访问权限
- [ ] 更新依赖
- [ ] 检查告警
- [ ] 审查安全事件

### 定期检查（每月）

- [ ] 完整安全审计
- [ ] 渗透测试
- [ ] 密钥轮换
- [ ] 权限审查
- [ ] 合规性检查
- [ ] 安全培训

## 事件响应

### 1. 安全事件分类

- **P0 - 紧急**: 数据泄露、系统入侵
- **P1 - 高**: 高危漏洞、未授权访问
- **P2 - 中**: 中危漏洞、可疑活动
- **P3 - 低**: 低危漏洞、策略违规

### 2. 响应流程

1. **检测**: 监控系统检测到异常
2. **分类**: 确定事件严重程度
3. **遏制**: 隔离受影响系统
4. **调查**: 分析事件原因
5. **修复**: 修复漏洞或问题
6. **恢复**: 恢复正常服务
7. **总结**: 编写事件报告

### 3. 联系方式

- **安全团队**: security@pixelcore.io
- **紧急热线**: +1-xxx-xxx-xxxx
- **事件报告**: https://pixelcore.io/security/report

## 最佳实践

1. **最小权限原则**: 只授予必要的权限
2. **深度防御**: 多层安全控制
3. **定期更新**: 及时更新依赖和补丁
4. **安全培训**: 定期进行安全培训
5. **自动化扫描**: CI/CD 集成安全扫描
6. **监控告警**: 实时监控安全事件
7. **事件演练**: 定期进行安全演练
8. **文档更新**: 保持安全文档最新

## 参考资源

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Falco Documentation](https://falco.org/docs/)

---

**最后更新**: 2026-03-06
**版本**: 1.0.0
**下次审查**: 2026-04-06
