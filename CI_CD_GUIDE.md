# CI/CD 流水线文档

本文档介绍 PixelCore 的 CI/CD 流水线配置和使用方法。

---

## 📋 概述

PixelCore 使用 GitHub Actions 实现完整的 CI/CD 流水线，包括：
- 代码质量检查
- 自动化测试
- Docker 镜像构建
- 安全扫描
- 自动部署
- 性能测试

---

## 🔄 工作流

### 1. CI/CD Pipeline (ci-cd.yml)

**触发条件**:
- Push 到 main 或 develop 分支
- Pull Request 到 main 或 develop 分支
- 创建 Release

**流程**:
```
代码质量检查 (lint)
    ↓
并行执行:
├─ 后端测试 (test-backend)
└─ 前端测试 (test-frontend)
    ↓
构建 Docker 镜像 (build-images)
    ↓
安全扫描 (security-scan)
    ↓
并行部署:
├─ 开发环境 (deploy-dev) [develop 分支]
└─ 生产环境 (deploy-prod) [main 分支]
    ↓
发送通知 (notify)
```

**任务详情**:

#### Lint (代码质量检查)
- Rust 格式检查 (`cargo fmt`)
- Rust Clippy 检查 (`cargo clippy`)
- TypeScript ESLint 检查

#### Test Backend (后端测试)
- 运行所有 Rust 测试
- 生成代码覆盖率报告
- 上传覆盖率到 Codecov

#### Test Frontend (前端测试)
- 运行前端测试
- 构建前端应用
- 上传构建产物

#### Build Images (构建镜像)
- 构建后端 Docker 镜像
- 构建前端 Docker 镜像
- 推送到 Docker Hub 和 GHCR
- 支持多平台 (amd64, arm64)
- 使用 GitHub Actions 缓存

#### Security Scan (安全扫描)
- 使用 Trivy 扫描镜像漏洞
- 上传结果到 GitHub Security

#### Deploy (部署)
- 开发环境: develop 分支自动部署
- 生产环境: main 分支自动部署
- 使用 SSH 连接到服务器
- 执行 docker-compose pull 和 up

---

### 2. Release (release.yml)

**触发条件**:
- 推送版本标签 (v*.*.*)

**流程**:
```
创建 GitHub Release
    ↓
并行执行:
├─ 构建多平台二进制文件
└─ 构建并推送 Docker 镜像
    ↓
发送发布通知
```

**支持平台**:
- Linux (amd64, arm64)
- macOS (amd64, arm64)
- Windows (amd64)

**产物**:
- 二进制文件上传到 GitHub Release
- Docker 镜像推送到 Docker Hub 和 GHCR
- 自动生成 Changelog

---

### 3. Code Quality (code-quality.yml)

**触发条件**:
- Push 到 main 或 develop 分支
- Pull Request
- 每天定时运行 (00:00 UTC)

**检查项**:
- Rust 代码格式和 Clippy
- TypeScript ESLint 和类型检查
- 依赖项过时检查
- 安全审计 (cargo audit, npm audit)
- 代码覆盖率
- 依赖关系审查
- CodeQL 静态分析

---

### 4. Performance (performance.yml)

**触发条件**:
- Push 到 main 分支
- Pull Request 到 main 分支
- 每周一定时运行 (00:00 UTC)

**测试项**:
- 负载测试 (k6)
- 基准测试 (cargo bench)
- 内存泄漏检测 (valgrind)

---

## 🔧 配置

### 必需的 Secrets

在 GitHub 仓库设置中配置以下 Secrets:

#### Docker Registry
```
DOCKER_USERNAME=your_docker_username
DOCKER_PASSWORD=your_docker_password
```

#### 部署服务器 (开发环境)
```
DEV_HOST=dev.example.com
DEV_USERNAME=deploy
DEV_SSH_KEY=<private_key>
```

#### 部署服务器 (生产环境)
```
PROD_HOST=prod.example.com
PROD_USERNAME=deploy
PROD_SSH_KEY=<private_key>
```

#### 通知
```
SLACK_WEBHOOK=https://hooks.slack.com/services/xxx
```

### 环境配置

在 GitHub 仓库设置中配置以下 Environments:

#### development
- URL: https://dev.pixelcore.example.com
- 保护规则: 无

#### production
- URL: https://pixelcore.example.com
- 保护规则:
  - 需要审批
  - 仅 main 分支可部署

---

## 📊 工作流状态

### 查看状态

在 GitHub 仓库页面查看工作流状态:
- Actions 标签页
- README 徽章

### 徽章示例

```markdown
![CI/CD](https://github.com/your-org/pixelcore/workflows/CI%2FCD%20Pipeline/badge.svg)
![Code Quality](https://github.com/your-org/pixelcore/workflows/Code%20Quality/badge.svg)
![Release](https://github.com/your-org/pixelcore/workflows/Release/badge.svg)
```

---

## 🚀 使用指南

### 开发流程

1. **创建功能分支**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **开发和提交**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin feature/my-feature
   ```

3. **创建 Pull Request**
   - 自动触发 CI/CD 流水线
   - 运行代码质量检查和测试
   - 等待 CI 通过

4. **合并到 develop**
   - 自动部署到开发环境
   - 验证功能

5. **合并到 main**
   - 自动部署到生产环境
   - 需要审批

### 发布流程

1. **更新版本号**
   ```bash
   # 更新 Cargo.toml 和 package.json 中的版本号
   vim Cargo.toml
   vim app/package.json
   ```

2. **创建版本标签**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

3. **自动发布**
   - 自动构建二进制文件
   - 自动构建 Docker 镜像
   - 自动创建 GitHub Release
   - 自动生成 Changelog

---

## 🔍 故障排查

### CI 失败

1. **查看日志**
   - 点击失败的工作流
   - 查看详细日志
   - 定位错误原因

2. **常见问题**
   - 代码格式问题: 运行 `cargo fmt` 和 `npm run lint`
   - 测试失败: 本地运行 `cargo test` 和 `npm test`
   - 构建失败: 检查依赖项和配置

### 部署失败

1. **检查服务器连接**
   ```bash
   ssh deploy@prod.example.com
   ```

2. **检查 Docker 服务**
   ```bash
   docker-compose ps
   docker-compose logs
   ```

3. **手动部署**
   ```bash
   cd /opt/pixelcore
   docker-compose pull
   docker-compose up -d
   ```

### 镜像构建失败

1. **本地测试**
   ```bash
   docker build -t pixelcore-backend:test .
   docker build -t pixelcore-frontend:test ./app
   ```

2. **检查 Dockerfile**
   - 验证基础镜像
   - 检查依赖安装
   - 验证构建步骤

---

## 📈 性能优化

### 缓存优化

- 使用 GitHub Actions 缓存
- 缓存 Cargo 依赖
- 缓存 npm 依赖
- 缓存 Docker 层

### 并行执行

- 后端和前端测试并行
- 多平台构建并行
- 独立任务并行

### 构建优化

- 多阶段 Docker 构建
- 依赖缓存层
- 增量构建

---

## 🔒 安全最佳实践

### Secrets 管理

- 使用 GitHub Secrets 存储敏感信息
- 不要在代码中硬编码密钥
- 定期轮换密钥

### 镜像安全

- 使用官方基础镜像
- 定期更新基础镜像
- 扫描漏洞 (Trivy)
- 最小化镜像大小

### 部署安全

- 使用 SSH 密钥认证
- 限制部署权限
- 启用环境保护规则
- 审计部署日志

---

## 📚 参考资料

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
- [Trivy 文档](https://aquasecurity.github.io/trivy/)
- [k6 文档](https://k6.io/docs/)

---

## 🆘 获取帮助

如果遇到问题:

1. 查看工作流日志
2. 查看本文档的故障排查部分
3. 提交 Issue: https://github.com/your-org/pixelcore/issues

---

**CI/CD 愉快！** 🚀
