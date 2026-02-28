# PixelCore 快速开始指南

## 🚀 5 分钟快速体验

### 1. 测试新增的 Skills

```bash
# 测试计算和数据处理 Skills
cargo run --example test_new_skills
```

**你会看到**:
- ✅ 数学计算：2 + 2 * 3 = 8
- ✅ 单位转换：100 cm = 1 m，32 F = 0 C
- ✅ JSON 解析和查询
- ✅ CSV 解析

### 2. 测试 MCP 服务器

```bash
# 测试所有 MCP 服务器（无需 API Key）
cargo run --example complete_mcp_demo
```

**你会看到**:
- ✅ 启动 3 个 MCP 服务器
- ✅ 发现 14 个工具
- ✅ 测试文件系统和时间工具

### 3. 测试 Agent 集成（需要 API Key）

```bash
# 设置 API Key
export SILICONFLOW_API_KEY=your-api-key

# 运行完整示例
cargo run --example complete_mcp_demo
```

**你会看到**:
- ✅ Agent 创建并注册 MCP 工具
- ✅ Agent 使用工具完成任务
- ✅ 3 个测试场景演示

---

## 📚 详细文档

### 核心文档
- [今日工作总结](TODAY_SUMMARY_2026-02-28.md) - 今天完成的所有工作
- [项目进度更新](PROGRESS_UPDATE_2026-02-28.md) - 项目当前状态和下一步计划

### MCP 相关
- [MCP 服务器集合](MCP_SERVERS_COLLECTION.md) - MCP 服务器详细文档
- [MCP 快速参考](MCP_QUICK_REFERENCE.md) - MCP 使用快速参考
- [完整 MCP 示例](examples/COMPLETE_MCP_DEMO.md) - Agent 集成示例文档

### Skills 相关
- [新增 Skills 报告](NEW_SKILLS_COMPLETE_2026-02-28.md) - 计算和数据处理 Skills 文档

---

## 🎯 当前项目状态

### 完成度: 80% ✅

#### 已完成的模块
- ✅ Agent Runtime（核心运行时）
- ✅ Swarm Engine（多 Agent 编排）
- ✅ Heartbeat（心流状态机）
- ✅ Storage（加密存储）
- ✅ Skills（24 个工具）
- ✅ MCP Runtime（本地 MCP 运行时）
- ✅ Tauri UI（基础框架）

#### 待完成的工作
- ⏳ Tauri UI 与后端集成
- ⏳ 性能优化和测试
- ⏳ 文档完善

---

## 🛠️ 开发命令

### 编译和测试
```bash
# 检查所有代码
cargo check --all-targets

# 编译所有目标
cargo build --all-targets

# 运行测试
cargo test

# 运行特定示例
cargo run --example <example_name>
```

### 可用的示例程序
- `test_new_skills` - 测试新增的计算和数据处理 Skills
- `complete_mcp_demo` - 完整的 MCP 集成示例
- `test_all_servers` - 测试所有 MCP 服务器
- `mcp_skills_test` - MCP Skills 基础测试
- `local_mcp_demo` - 本地 MCP 基础示例

---

## 📊 项目统计

### 代码量
- **Rust 代码**: ~16,000 行
- **Python 代码**: ~500 行
- **TypeScript 代码**: ~1,000 行
- **文档**: ~5,000 行

### 模块数量
- **Crates**: 9 个
- **MCP 服务器**: 3 个
- **Skills**: 24 个
- **示例程序**: 10+ 个

### Skills 分类
| 类别 | 数量 | Skills |
|------|------|--------|
| MCP - 文件系统 | 5 | read_file, write_file, list_dir, file_exists, get_file_info |
| MCP - HTTP | 4 | http_get, http_post, http_put, http_delete |
| MCP - 时间 | 5 | get_current_time, format_time, parse_time, time_diff, add_time |
| 基础 | 5 | echo, storage_get, storage_set, http_fetch, delegate |
| 计算 | 2 | calculate, convert_units |
| 数据处理 | 3 | json_parse, json_query, csv_parse |
| **总计** | **24** | |

---

## 🎓 学习资源

### 架构理解
1. 阅读 `MCP_SERVERS_COLLECTION.md` 了解 MCP 架构
2. 查看 `examples/complete_mcp_demo.rs` 了解 Agent 集成
3. 阅读 `NEW_SKILLS_COMPLETE_2026-02-28.md` 了解 Skills 扩展

### 代码示例
1. **创建 MCP 服务器**: 参考 `examples/mcp_servers/`
2. **使用 MCP 工具**: 参考 `examples/complete_mcp_demo.rs`
3. **创建新 Skill**: 参考 `crates/pixelcore-skills/src/builtins/compute.rs`
4. **Agent 集成**: 参考 `examples/complete_mcp_demo.rs`

---

## 🐛 故障排除

### 问题 1: MCP 服务器启动失败
```
⚠️  HTTP 服务器启动失败
```
**解决方案**: 安装依赖
```bash
pip install requests
```

### 问题 2: 编译错误
```
error: could not compile `pixelcore`
```
**解决方案**: 清理并重新编译
```bash
cargo clean
cargo build
```

### 问题 3: 测试失败
```
test result: FAILED
```
**解决方案**: 单独运行失败的测试
```bash
cargo test <test_name> -- --nocapture
```

---

## 💡 下一步建议

### 如果你想...

#### 1. 了解项目整体进度
👉 阅读 [项目进度更新](PROGRESS_UPDATE_2026-02-28.md)

#### 2. 使用 MCP 工具
👉 阅读 [MCP 快速参考](MCP_QUICK_REFERENCE.md)
👉 运行 `cargo run --example complete_mcp_demo`

#### 3. 创建新的 Skill
👉 阅读 [新增 Skills 报告](NEW_SKILLS_COMPLETE_2026-02-28.md)
👉 参考 `crates/pixelcore-skills/src/builtins/compute.rs`

#### 4. 集成到你的应用
👉 阅读 [完整 MCP 示例文档](examples/COMPLETE_MCP_DEMO.md)
👉 查看 `examples/complete_mcp_demo.rs` 源码

#### 5. 贡献代码
👉 查看 [项目进度更新](PROGRESS_UPDATE_2026-02-28.md) 中的"待完成工作"
👉 选择一个任务开始实现

---

## 🎉 今日亮点

### 上午完成
- ✅ 本地 MCP 运行时（100%）
- ✅ 3 个 MCP 服务器（14 个工具）
- ✅ Agent 集成示例

### 下午完成
- ✅ 2 个计算 Skills
- ✅ 3 个数据处理 Skills
- ✅ 完整的测试和文档

### 总计
- 📝 18 个新文件
- 💻 ~3,300 行代码和文档
- 🎯 项目完成度从 75% 提升到 80%
- 🚀 Skills 从 5 个增加到 24 个（增长 380%）

---

## 📞 获取帮助

### 文档
- 所有文档都在项目根目录
- 以 `.md` 结尾的文件都是文档
- 按日期命名的文档是当天的工作报告

### 示例代码
- 所有示例都在 `examples/` 目录
- 运行 `cargo run --example <name>` 查看效果

### 源代码
- 核心代码在 `crates/` 目录
- 每个 crate 都有独立的功能
- 查看 `src/lib.rs` 了解模块结构

---

## 🎊 开始使用

```bash
# 1. 克隆项目（如果还没有）
git clone <repository-url>
cd pixelcore

# 2. 安装依赖
pip install requests  # Python 依赖

# 3. 运行测试
cargo run --example test_new_skills

# 4. 体验 MCP
cargo run --example complete_mcp_demo

# 5. 开始开发
# 查看 PROGRESS_UPDATE_2026-02-28.md 了解下一步工作
```

祝你使用愉快！🚀
