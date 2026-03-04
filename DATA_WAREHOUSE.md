# 数据仓库文档

## 📋 概述

PixelCore 数据仓库提供完整的数据分析基础设施，包括数据存储、ETL 流程、指标收集和查询能力。

## 🎯 核心功能

### 1. 数据仓库 (Data Warehouse)

基于 PostgreSQL 的高性能数据仓库：

- **事件存储**: 存储用户行为事件
- **指标存储**: 存储系统和业务指标
- **聚合表**: 预计算的聚合数据
- **时序索引**: 优化时间范围查询

### 2. ETL 流程 (ETL Pipeline)

灵活的数据提取、转换、加载流程：

- **多源支持**: 数据库、API、文件、流
- **转换规则**: Map、Filter、Aggregate、Join
- **调度执行**: Cron 表达式调度
- **状态跟踪**: 实时任务状态监控

### 3. 指标收集 (Metrics Collector)

Prometheus 兼容的指标收集：

- **事件计数**: 处理的事件总数
- **处理时长**: 事件处理耗时
- **仓库大小**: 数据仓库存储大小
- **ETL 任务**: 运行中的 ETL 任务数

## 🏗️ 架构设计

```
┌─────────────────────────────────────────┐
│       Data Warehouse                    │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐  ┌──────────────┐   │
│  │   Events     │  │   Metrics    │   │
│  │   Table      │  │   Table      │   │
│  │              │  │              │   │
│  │ - event_type │  │ - metric_name│   │
│  │ - user_id    │  │ - value      │   │
│  │ - data       │  │ - labels     │   │
│  │ - timestamp  │  │ - timestamp  │   │
│  └──────────────┘  └──────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │    Aggregated Metrics            │  │
│  │    (Pre-computed)                │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│       ETL Pipeline                      │
├─────────────────────────────────────────┤
│                                         │
│  Extract → Transform → Load             │
│                                         │
│  Sources:                               │
│  - Database                             │
│  - API                                  │
│  - File                                 │
│  - Stream                               │
└─────────────────────────────────────────┘
```

## 🚀 使用指南

### 1. 初始化数据仓库

```rust
use pixelcore_analytics::{DataWarehouse, WarehouseConfig};

let config = WarehouseConfig {
    host: "localhost".to_string(),
    port: 5432,
    dbname: "pixelcore_analytics".to_string(),
    user: "postgres".to_string(),
    password: "postgres".to_string(),
    pool_size: 10,
};

let warehouse = DataWarehouse::new(config).await?;
warehouse.initialize().await?;
```

### 2. 插入事件

```rust
let event_id = warehouse.insert_event(
    "page_view",
    Some(user_id),
    serde_json::json!({
        "page": "/home",
        "duration": 2.5
    })
).await?;
```

### 3. 插入指标

```rust
let metric_id = warehouse.insert_metric(
    "response_time_ms",
    45.2,
    Some(serde_json::json!({"region": "us-east"}))
).await?;
```

### 4. 查询事件

```rust
let events = warehouse.query_events(
    Some("page_view"),  // event_type
    None,               // user_id
    None,               // start_time
    None,               // end_time
    10                  // limit
).await?;
```

### 5. 创建 ETL 任务

```rust
use pixelcore_analytics::{EtlJob, etl::*};

let config = EtlJobConfig {
    name: "database_sync".to_string(),
    source_type: SourceType::Database {
        connection_string: "postgresql://localhost/source_db".to_string(),
    },
    transform_rules: vec![
        TransformRule {
            rule_type: TransformType::Filter,
            field: "status".to_string(),
            params: serde_json::json!({"value": "active"}),
        },
    ],
    schedule: Some("0 * * * *".to_string()),
};

let job = EtlJob::new(config, warehouse);
job.run().await?;
```

### 6. 收集指标

```rust
use pixelcore_analytics::MetricsCollector;

let metrics = MetricsCollector::new();

metrics.inc_events();
metrics.record_processing_duration(0.5);
metrics.set_warehouse_size(1024.0);

// Export metrics
let metrics_text = metrics.metrics_text();
```

## 📊 性能指标

### 目标指标

- **数据同步延迟**: < 5 秒
- **数据准确率**: > 99.9%
- **查询响应时间**: < 100ms
- **写入吞吐量**: > 10,000 events/sec
- **存储容量**: 支持 PB 级数据

## 🧪 测试

### 运行单元测试

```bash
cd crates/pixelcore-analytics
cargo test
```

### 运行示例程序

```bash
# 启动 PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres

# 运行演示
cargo run --example analytics_demo
```

## 🔧 配置

### 环境变量

```bash
# PostgreSQL 连接
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres

# 连接池大小
WAREHOUSE_POOL_SIZE=10
```

## 📈 未来改进

### 短期
- [ ] 实时流处理
- [ ] 更多聚合函数
- [ ] 查询优化器

### 中期
- [ ] 分布式查询
- [ ] 数据分区
- [ ] 自动备份

### 长期
- [ ] 机器学习集成
- [ ] 预测分析
- [ ] 异常检测

## 📝 更新日志

### v0.1.0 (2026-03-04)

- ✅ 实现数据仓库核心功能
- ✅ 实现 ETL 流程
- ✅ 实现指标收集
- ✅ 添加单元测试
- ✅ 创建示例程序
