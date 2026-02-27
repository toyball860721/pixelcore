# PixelCore

全球一人公司 Agent-to-Agent 商业交易平台

## 快速开始

### 1. 环境配置

复制 `.env.example` 文件并重命名为 `.env`：

```bash
cp .env.example .env
```

编辑 `.env` 文件，填入你的 API key：

```env
SILICONFLOW_API_KEY=sk-your-api-key-here
MODEL_NAME=Pro/MiniMaxAI/MiniMax-M2.5
```

获取 SiliconFlow API Key：https://cloud.siliconflow.cn/

### 2. 编译运行

```bash
# 编译
cargo build

# 运行主程序
cargo run

# 运行心流状态机示例
cargo run --package pixelcore-heartbeat --example flow_direct
cargo run --package pixelcore-heartbeat --example flow_demo
```

## 项目结构

```
pixelcore/
├── crates/
│   ├── pixelcore-runtime/     # Agent 运行时
│   ├── pixelcore-swarm/       # Swarm 编排
│   ├── pixelcore-heartbeat/   # 心跳/心流机制
│   ├── pixelcore-claw/        # MCP 客户端
│   ├── pixelcore-skills/      # Skills 执行器
│   ├── pixelcore-storage/     # 持久化存储
│   ├── pixelcore-ipc/         # IPC 通信
│   └── pixelcore-agents/      # Agent 实现
└── src/
    └── main.rs                # 主程序入口
```

## 已实现功能

- ✅ Agent 运行时核心
- ✅ 持久化存储（sled）
- ✅ Skills 系统（echo, storage_get, storage_set）
- ✅ Swarm 编排
- ✅ 心流状态机（Idle → Working → DeepFlow → Hyperfocus）
- ✅ EventBus 广播系统
- ✅ SiliconFlow MiniMax 集成

## 安全说明

⚠️ **重要**:
- 不要将 `.env` 文件提交到 Git
- 不要在代码中硬编码 API key
- 使用环境变量管理敏感信息

## 开发路线图

详见 [PIXEL_PLAN_ROADMAP.md](../PIXEL_PLAN_ROADMAP.md)

## License

MIT
