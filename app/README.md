# PixelCore Desktop App

基于 Tauri 2.0 的 PixelCore 桌面应用。

## 技术栈

- **后端**: Rust + Tauri 2.0
- **前端**: React + TypeScript + Vite
- **状态管理**: Tauri State Management

## 开发环境设置

### 前置要求

1. Rust (最新稳定版)
2. Node.js (v18+)
3. Tauri CLI

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Tauri CLI (如果还没安装)
cargo install tauri-cli --version "^2.0.0"
```

## 运行应用

### 开发模式

```bash
# 启动开发服务器
cargo tauri dev
```

### 构建生产版本

```bash
# 构建应用
cargo tauri build
```

## 功能特性

- Agent 状态实时监控
- Agent 启动/停止控制
- 心流等级显示
- 任务状态跟踪

## 项目结构

```
app/
├── src/              # React 前端代码
│   ├── App.tsx       # 主应用组件
│   ├── main.tsx      # 入口文件
│   └── *.css         # 样式文件
├── src-tauri/        # Tauri 后端代码
│   ├── src/
│   │   └── main.rs   # Rust 主程序
│   ├── Cargo.toml    # Rust 依赖
│   └── tauri.conf.json  # Tauri 配置
└── package.json      # Node.js 依赖
```
