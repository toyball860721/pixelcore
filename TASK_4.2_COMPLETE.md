# Task 4.2: 日志和追踪 - COMPLETE ✅

## 完成时间
2026-03-03

## 实现内容

### 1. 核心模块

#### 1.1 数据模型 (models.rs)
- **日志级别**: Trace, Debug, Info, Warn, Error
- **日志记录**: 时间戳、级别、目标、消息、字段、span_id、trace_id
- **Span**: 追踪单元，包含开始/结束时间、持续时间、字段、事件
- **Trace**: 完整调用链，包含多个 span
- **日志查询**: 支持按级别、目标、消息、trace_id、时间范围查询
- **日志统计**: 按级别、目标统计，时间范围分析

#### 1.2 日志管理器 (logger.rs)
- 结构化日志记录
- 5 个日志级别 (trace, debug, info, warn, error)
- 支持自定义字段
- 自动关联 trace 和 span 上下文
- 日志查询和过滤
- 日志统计分析
- JSON 格式导出

#### 1.3 追踪管理器 (tracer.rs)
- Trace 生命周期管理
- Span 创建和嵌套
- 父子关系追踪
- Span 字段和事件
- 自动计算持续时间
- 性能分析支持

### 2. 核心特性

#### 2.1 结构化日志
- JSON 格式输出
- 支持自定义字段
- 上下文信息 (target, fields)
- 时间戳自动记录
- 唯一 ID 标识

#### 2.2 日志聚合
- 内存存储 (可扩展到文件/数据库)
- 灵活的查询 API
- 多条件过滤
- 统计分析
- 时间范围查询

#### 2.3 分布式追踪
- Trace ID 传播
- Span 嵌套和父子关系
- 性能指标自动收集
- 调用链可视化
- 事件记录

### 3. 测试覆盖

#### 单元测试 (13个测试全部通过)

**Logger 测试 (6个)**:
- `test_log_basic`: 基础日志记录
- `test_log_with_fields`: 带字段的日志
- `test_query_by_level`: 按级别查询
- `test_query_by_message`: 按消息查询
- `test_stats`: 日志统计
- `test_with_trace_context`: 追踪上下文

**Tracer 测试 (7个)**:
- `test_create_trace`: 创建 trace
- `test_create_span`: 创建 span
- `test_nested_spans`: 嵌套 span
- `test_span_fields`: Span 字段
- `test_span_events`: Span 事件
- `test_span_duration`: 持续时间计算
- `test_trace_finish`: Trace 完成

### 4. 示例程序

#### logging_demo.rs
演示完整的日志和追踪功能:
1. 基础日志记录 (5个级别)
2. 结构化日志 (带字段)
3. 日志查询 (按级别、消息、目标)
4. 日志统计 (按级别、目标)
5. 分布式追踪 (trace + spans)
6. 日志与追踪集成
7. 性能分析
8. JSON 导出
9. 数据清理

### 5. 技术特性

- **异步支持**: 使用 tokio 异步运行时
- **线程安全**: Arc<Mutex<>> 保证并发安全
- **类型安全**: 强类型系统，编译时检查
- **零拷贝**: 高效的数据结构设计
- **可扩展**: 易于添加新的存储后端
- **标准兼容**: 使用 tracing crate 生态

### 6. 使用示例

```rust
use pixelcore_logging::{Logger, Tracer, LogLevel};

// 创建日志管理器
let logger = Logger::new();

// 记录日志
logger.info("app".to_string(), "Application started".to_string());

// 带字段的日志
let mut fields = HashMap::new();
fields.insert("user_id".to_string(), "123".to_string());
logger.log_with_fields(
    LogLevel::Info,
    "auth".to_string(),
    "User logged in".to_string(),
    fields,
);

// 创建追踪
let tracer = Tracer::new();
let trace_id = tracer.start_trace("api_call".to_string());
let span_id = tracer.start_span(trace_id, "process".to_string(), None);

// 关联日志和追踪
logger.set_current_trace(Some(trace_id));
logger.set_current_span(Some(span_id));
logger.info("api".to_string(), "Processing request".to_string());

// 结束追踪
tracer.end_span(span_id);
tracer.end_trace(trace_id);

// 查询日志
let query = LogQuery::new()
    .with_level(LogLevel::Info)
    .with_trace_id(trace_id);
let logs = logger.query(&query);

// 获取统计
let stats = logger.get_stats();
println!("Total logs: {}", stats.total_logs);
```

### 7. 文件清单

```
crates/pixelcore-logging/
├── Cargo.toml                 # 依赖配置
├── src/
│   ├── lib.rs                 # 模块导出
│   ├── models.rs              # 数据模型
│   ├── logger.rs              # 日志管理器
│   └── tracer.rs              # 追踪管理器
examples/
└── logging_demo.rs            # 演示程序
```

### 8. 依赖项

- tokio: 异步运行时
- serde: 序列化/反序列化
- serde_json: JSON 支持
- chrono: 时间处理
- uuid: 唯一标识符
- thiserror: 错误处理
- tracing: 日志和追踪框架
- tracing-subscriber: 日志订阅器

## 测试结果

```bash
$ cargo test -p pixelcore-logging
running 13 tests
test logger::tests::test_log_basic ... ok
test logger::tests::test_log_with_fields ... ok
test logger::tests::test_query_by_level ... ok
test logger::tests::test_query_by_message ... ok
test logger::tests::test_stats ... ok
test logger::tests::test_with_trace_context ... ok
test tracer::tests::test_create_trace ... ok
test tracer::tests::test_create_span ... ok
test tracer::tests::test_nested_spans ... ok
test tracer::tests::test_span_fields ... ok
test tracer::tests::test_span_events ... ok
test tracer::tests::test_span_duration ... ok
test tracer::tests::test_trace_finish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored
```

```bash
$ cargo run --example logging_demo
=== PixelCore Logging and Tracing Demo ===

1. Basic Logging
  Logged 4 messages

2. Structured Logging with Fields
  Logged structured message with 3 fields

3. Log Querying
  INFO logs: 2
  Logs containing 'Error': 1
  Auth logs: 1

4. Log Statistics
  Total logs: 5
  By level: INFO: 2, DEBUG: 1, WARN: 1, ERROR: 1
  By target: app: 4, auth: 1

5. Distributed Tracing
  Started trace: 286926bc-9fc2-4257-8adf-8604f48ae40b
  Trace completed with 3 spans

6. Logging with Trace Context
  Logs with trace context: 2

7. Performance Analysis
  Trace: user_request
  Total duration: 101 ms
  Spans:
    - database_query: 34 ms
    - cache_operation: 11 ms
    - handle_request: 101 ms

8. JSON Export
  Sample log as JSON:
  {"id":"...","timestamp":"...","level":"Info","target":"app","message":"Application started","fields":{},"span_id":null,"trace_id":null}

9. Cleanup
  Cleared 5 logs (before: 5, after: 0)
  Cleared 2 traces (before: 2, after: 0)

=== Demo Complete ===
```

## 性能特点

- **低开销**: 内存存储，快速查询
- **高并发**: 线程安全的并发访问
- **可扩展**: 支持大量日志和追踪
- **实时性**: 即时记录和查询

## 扩展方向

1. **持久化存储**: 支持文件、数据库存储
2. **日志轮转**: 自动归档和清理
3. **远程传输**: 发送到集中式日志系统
4. **OpenTelemetry**: 完整的 OTEL 集成
5. **可视化**: Grafana/Jaeger 集成
6. **采样**: 高流量下的采样策略

## 下一步

Task 4.2 (日志和追踪) 已完成 ✅

继续 Phase 3 Week 7-8 的其他任务:
- Task 4.3: 备份和恢复 (Backup and Recovery)
- Task 4.4: 开发者生态 (Developer Ecosystem)
- Task 4.5: UI 增强 (UI Enhancements)
