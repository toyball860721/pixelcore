# Tauri UI 编译指南

## 当前状态

Tauri UI 的所有代码已经完成，但在编译时遇到了内存问题（构建脚本被 SIGKILL）。

## 解决方案

### 方案 1：增加系统内存（推荐）

如果可能，增加系统可用内存，然后重新编译：

```bash
cd /Users/toyball/Desktop/ClaudeUse/pixelcore
cargo build -p pixelcore-app
```

### 方案 2：使用 Release 模式

Release 模式编译通常使用更少的内存：

```bash
cargo build --release -p pixelcore-app
```

### 方案 3：分步编译

先编译依赖，再编译 Tauri 应用：

```bash
# 1. 编译所有依赖
cargo build -p pixelcore-runtime
cargo build -p pixelcore-agents
cargo build -p pixelcore-skills
cargo build -p pixelcore-claw

# 2. 编译 Tauri 应用
cargo build -p pixelcore-app
```

### 方案 4：清理并重新编译

```bash
# 清理构建缓存
cargo clean

# 重新编译
cargo build -p pixelcore-app
```

## 启动 Tauri 应用

编译成功后，启动应用：

```bash
cd app
npm install  # 首次运行需要安装前端依赖
npm run tauri dev
```

## 使用说明

### 1. 创建 Agent

1. 点击 "New Agent" 按钮
2. 填写 Agent 信息：
   - Name: Agent 名称
   - Model: 选择模型（DeepSeek V3, Qwen 2.5, Llama 3.3）
   - System Prompt: 系统提示词
   - API Key: SiliconFlow API Key
3. 点击 "Create Agent"

### 2. 与 Agent 对话

1. 从侧边栏选择一个 Agent
2. 在输入框中输入消息
3. 按 Enter 或点击 "Send" 发送
4. Agent 会使用 24 个 Skills 来回答你的问题

### 3. 可用的 Skills

Agent 可以使用以下 Skills：

**计算 Skills**:
- `calculate` - 计算数学表达式
- `convert_units` - 单位转换

**数据处理 Skills**:
- `json_parse` - 解析 JSON
- `json_query` - 查询 JSON 数据
- `csv_parse` - 解析 CSV

**MCP Skills**（如果启动了 MCP 服务器）:
- 文件系统操作（5 个工具）
- HTTP 请求（4 个工具）
- 时间处理（5 个工具）

### 4. 示例对话

```
User: Calculate 2 + 2 * 3
Agent: [使用 calculate skill] The result is 8.

User: Convert 100 cm to meters
Agent: [使用 convert_units skill] 100 cm equals 1.0 m.

User: Parse this JSON: {"name": "Alice", "age": 30}
Agent: [使用 json_parse skill] The JSON contains:
- name: Alice
- age: 30
```

## 故障排除

### 问题 1: 编译失败（SIGKILL）
**原因**: 内存不足
**解决**: 使用上述方案 1-4

### 问题 2: 前端依赖安装失败
**解决**:
```bash
cd app
rm -rf node_modules package-lock.json
npm install
```

### 问题 3: API Key 错误
**解决**: 确保使用有效的 SiliconFlow API Key

### 问题 4: Agent 创建失败
**解决**: 检查网络连接和 API Key

## 开发模式

### 前端开发
```bash
cd app
npm run dev  # 启动 Vite 开发服务器
```

### 后端开发
```bash
cargo watch -x "build -p pixelcore-app"
```

### 同时开发前后端
```bash
cd app
npm run tauri dev
```

## 生产构建

```bash
cd app
npm run tauri build
```

构建产物位于 `app/src-tauri/target/release/`

## 下一步

一旦编译成功，你就可以：
1. ✅ 通过 UI 创建 Agent
2. ✅ 与 Agent 对话
3. ✅ Agent 使用 24 个 Skills
4. ✅ 查看对话历史
5. ✅ 管理多个 Agent

享受使用 PixelCore！🎉
