# PixelCore 下一步工作计划

## 当前状态评估

### ✅ 已完成（Phase 1: Week 1-8）
- Week 1-2: 项目脚手架、Rust 工作区、CI/CD ✅
- Week 3-4: Agent Runtime 核心数据结构 + 状态机 ✅
- Week 5-6: Swarm Engine + 任务调度 ✅
- Week 7-8: 心跳/心流机制 ✅
- 额外完成：
  - SQLCipher 加密存储 ✅
  - Tauri UI 基础框架 ✅
  - 集成测试框架 ✅

### 🔄 进行中的模块
- pixelcore-runtime: 核心运行时 ✅
- pixelcore-swarm: Swarm 编排 ✅
- pixelcore-heartbeat: 心流状态机 ✅
- pixelcore-storage: 加密存储 ✅
- pixelcore-agents: ClaudeAgent 实现 ✅
- pixelcore-skills: 基础 Skills (echo, storage_get, storage_set) ✅
- pixelcore-claw: MCP 客户端（SiliconFlow 集成）✅
- pixelcore-ipc: IPC 通信 ⚠️（基础实现）

### ❌ 未完成的模块
- python/: Python Agent 逻辑层（未创建）
- 本地 MCP 运行时（pixelcore-claw 只支持远程 API）
- Skills 执行器完善（只有 3 个基础 skill）
- Tauri UI 与后端集成（UI 和 Runtime 未连接）

---

## 下一步工作计划（优先级排序）

### 🎯 优先级 1：完善核心功能（Week 9-10）

#### 1.1 本地 MCP 运行时实现
**目标**: 支持本地 MCP 服务器，而不仅仅是远程 API

**任务清单**:
- [ ] 在 pixelcore-claw 中添加本地 MCP 服务器支持
- [ ] 实现 stdio 传输协议（用于本地进程通信）
- [ ] 实现 MCP 工具发现和调用机制
- [ ] 添加本地 MCP 服务器管理（启动/停止/重启）
- [ ] 创建示例本地 MCP 服务器（文件系统、数据库等）

**预期产出**:
- `crates/pixelcore-claw/src/local_mcp.rs` - 本地 MCP 实现
- `crates/pixelcore-claw/src/stdio_transport.rs` - stdio 传输
- `examples/local_mcp_server/` - 示例 MCP 服务器

**技术要点**:
- 使用 tokio::process 管理子进程
- 实现 JSON-RPC over stdio
- 支持工具列表、工具调用、工具结果返回

---

#### 1.2 Skills 执行器扩展
**目标**: 添加更多实用的 Skills，完善 Skills 系统

**任务清单**:
- [ ] 文件系统 Skills（read_file, write_file, list_dir）
- [ ] HTTP 请求 Skills（http_get, http_post）
- [ ] 时间/日期 Skills（get_time, schedule_task）
- [ ] 计算 Skills（calculate, convert_units）
- [ ] 数据处理 Skills（json_parse, csv_parse）
- [ ] Skill 权限管理系统
- [ ] Skill 执行超时和错误处理

**预期产出**:
- `crates/pixelcore-skills/src/filesystem.rs`
- `crates/pixelcore-skills/src/http.rs`
- `crates/pixelcore-skills/src/time.rs`
- `crates/pixelcore-skills/src/compute.rs`
- `crates/pixelcore-skills/src/data.rs`
- `crates/pixelcore-skills/src/permissions.rs`

**技术要点**:
- 使用 tokio::fs 实现异步文件操作
- 使用 reqwest 实现 HTTP 请求
- 使用 chrono 处理时间
- 实现 Skill 沙箱机制（限制文件访问范围等）

---

#### 1.3 Python Agent 逻辑集成（可选）
**目标**: 通过 PyO3 嵌入 Python，支持 Python 编写的 Agent 逻辑

**任务清单**:
- [ ] 添加 PyO3 依赖
- [ ] 创建 Python 绑定层
- [ ] 实现 Python Agent 接口
- [ ] 创建 Python Agent 示例
- [ ] 添加 Python 环境管理

**预期产出**:
- `crates/pixelcore-python/` - Python 绑定 crate
- `python/` - Python Agent 代码目录
- `python/examples/` - Python Agent 示例

**技术要点**:
- 使用 PyO3 创建 Rust-Python 绑定
- 实现 Python 调用 Rust Skills
- 实现 Rust 调用 Python Agent 逻辑
- 处理 Python GIL 和异步问题

**注意**: 这个任务可以延后，因为当前 Rust Agent 已经足够强大

---

### 🎯 优先级 2：Tauri UI 与后端集成（Week 11）

#### 2.1 连接 Tauri UI 与 PixelCore Runtime
**目标**: 让 Tauri UI 能够真实地控制和监控 Agent

**任务清单**:
- [ ] 在 Tauri 后端集成 PixelCore Runtime
- [ ] 实现真实的 Agent 创建和管理
- [ ] 实现 Agent 状态实时更新（使用 EventBus）
- [ ] 实现 Agent 日志流式传输到 UI
- [ ] 添加 Agent 配置界面
- [ ] 添加 Skills 管理界面

**预期产出**:
- 更新 `app/src-tauri/src/main.rs` - 集成 Runtime
- 新增 `app/src-tauri/src/agent_manager.rs` - Agent 管理器
- 新增 `app/src-tauri/src/event_bridge.rs` - 事件桥接
- 更新前端组件以显示真实数据

**技术要点**:
- 使用 Tauri State 管理 Runtime 实例
- 使用 Tauri Events 实现实时更新
- 实现 WebSocket 或 SSE 用于日志流

---

#### 2.2 UI 功能增强
**目标**: 添加更多可视化和交互功能

**任务清单**:
- [ ] Agent 配置编辑器（JSON 编辑器）
- [ ] 心流状态可视化（图表）
- [ ] 任务历史记录查看器
- [ ] Skills 执行日志查看器
- [ ] 存储数据浏览器（查看加密存储的内容）
- [ ] 系统设置页面（API Key 配置等）

**预期产出**:
- `app/src/components/AgentConfig.tsx`
- `app/src/components/FlowChart.tsx`
- `app/src/components/TaskHistory.tsx`
- `app/src/components/SkillLogs.tsx`
- `app/src/components/StorageBrowser.tsx`
- `app/src/components/Settings.tsx`

**技术要点**:
- 使用 React 状态管理（Context API 或 Zustand）
- 使用图表库（recharts 或 chart.js）
- 使用 Monaco Editor 实现 JSON 编辑器

---

### 🎯 优先级 3：性能优化和稳定性（Week 12）

#### 3.1 性能优化
**任务清单**:
- [ ] 添加性能基准测试
- [ ] 优化 Storage 性能（批量操作、缓存）
- [ ] 优化 EventBus 性能（减少锁竞争）
- [ ] 优化 Agent 消息处理（消息队列）
- [ ] 添加性能监控指标

**预期产出**:
- `benches/` - 性能基准测试
- 性能优化报告

---

#### 3.2 稳定性测试
**任务清单**:
- [ ] 24小时持续运行测试
- [ ] 内存泄漏检测（使用 valgrind 或 heaptrack）
- [ ] 压力测试（大量 Agent、大量消息）
- [ ] 错误恢复测试（崩溃恢复、数据恢复）
- [ ] 添加健康检查端点

**预期产出**:
- `tests/stress_test.rs` - 压力测试
- `tests/stability_test.rs` - 稳定性测试
- 稳定性测试报告

---

#### 3.3 文档完善
**任务清单**:
- [ ] API 文档（使用 rustdoc）
- [ ] 用户手册
- [ ] 开发者指南
- [ ] 架构设计文档
- [ ] 部署指南

**预期产出**:
- `docs/api/` - API 文档
- `docs/user-guide.md` - 用户手册
- `docs/developer-guide.md` - 开发者指南
- `docs/architecture.md` - 架构文档
- `docs/deployment.md` - 部署指南

---

## 推荐的实施顺序

### 第一周（Week 9）
1. **Day 1-2**: 实现本地 MCP 运行时基础
2. **Day 3-4**: 添加文件系统和 HTTP Skills
3. **Day 5**: 添加 Skills 权限管理

### 第二周（Week 10）
1. **Day 1-2**: 完善本地 MCP 运行时，添加示例服务器
2. **Day 3-4**: 添加更多 Skills（时间、计算、数据处理）
3. **Day 5**: Skills 系统测试和文档

### 第三周（Week 11）
1. **Day 1-2**: Tauri UI 与 Runtime 集成
2. **Day 3-4**: 实现 Agent 配置和管理界面
3. **Day 5**: 添加心流可视化和日志查看器

### 第四周（Week 12）
1. **Day 1-2**: 性能优化和基准测试
2. **Day 3-4**: 稳定性测试（24小时运行测试）
3. **Day 5**: 文档完善和发布准备

---

## 技术债务和改进建议

### 需要重构的部分
1. **EventBus**: 当前使用 broadcast channel，考虑改用更高效的实现
2. **Storage**: 考虑添加缓存层，减少磁盘 I/O
3. **Error Handling**: 统一错误处理策略，添加更详细的错误信息

### 需要添加的功能
1. **配置管理**: 统一的配置文件系统（TOML 或 YAML）
2. **插件系统**: 支持动态加载 Skills 和 Agents
3. **监控和告警**: 添加 Prometheus metrics 和告警机制
4. **备份和恢复**: 自动备份加密存储数据

---

## 资源需求

### 开发工具
- Rust 工具链（已有）
- Node.js 和 npm（已有）
- Python 3.8+（如果实现 Python 集成）
- Docker（用于测试和部署）

### 第三方服务
- SiliconFlow API（已有）
- 可选：其他 LLM API（OpenAI, Anthropic）

### 测试环境
- macOS（主要开发环境）
- Linux（CI/CD 和生产环境）
- Windows（可选，用于跨平台测试）

---

## 风险和挑战

### 技术风险
1. **PyO3 集成复杂度**: Python 集成可能遇到 GIL 和异步问题
2. **本地 MCP 稳定性**: 子进程管理可能不稳定
3. **性能瓶颈**: 大量 Agent 并发可能遇到性能问题

### 缓解措施
1. Python 集成可以延后，先专注于 Rust 实现
2. 充分测试本地 MCP，添加进程监控和自动重启
3. 早期进行性能测试，及时发现和解决瓶颈

---

## 成功标准

### Phase 1 完成标准
- [ ] 所有核心模块实现并测试通过
- [ ] Tauri UI 可以完整控制 Agent
- [ ] 24小时稳定运行测试通过
- [ ] 文档完善，可供其他开发者使用
- [ ] 性能达标（支持至少 10 个并发 Agent）

### 下一阶段准备
- [ ] 代码质量达标（clippy 无警告）
- [ ] 测试覆盖率 > 70%
- [ ] 安全审计通过
- [ ] 准备好进入 Phase 2（PixelVis）

---

## 总结

当前项目已经完成了 Phase 1 的大部分核心工作（Week 1-8），接下来的 4 周重点是：

1. **Week 9-10**: 完善 Skills 系统和本地 MCP 运行时
2. **Week 11**: Tauri UI 与后端深度集成
3. **Week 12**: 性能优化、稳定性测试和文档完善

建议优先实现 **本地 MCP 运行时** 和 **Skills 扩展**，因为这些是核心功能。Python 集成可以作为可选项延后实现。
