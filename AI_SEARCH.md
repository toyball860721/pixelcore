# AI 增强搜索文档

## 📋 概述

PixelCore AI 增强搜索系统提供高性能的全文搜索、智能排序、自动完成和搜索建议功能。

## 🎯 核心功能

### 1. 全文搜索 (Full-Text Search)

基于 Tantivy 的高性能全文搜索引擎：

- **倒排索引**: 快速文本检索
- **多字段搜索**: 同时搜索标题、内容、标签
- **模糊搜索**: 支持拼写错误容错
- **分页支持**: offset + limit 分页

**特点**:
- 响应时间 < 50ms
- 支持百万级文档
- 内存高效
- 实时索引更新

### 2. 智能排序 (Intelligent Ranking)

自定义排序算法提升搜索相关性：

- **标题权重**: 标题匹配权重 2x
- **精确匹配**: 完全匹配权重 3x
- **内容权重**: 内容匹配基础权重
- **时效性权重**: 新文档权重提升

**排序因子**:
- 文本相关性
- 字段权重
- 文档新鲜度
- 用户行为（未来）

### 3. 自动完成 (Autocomplete)

基于 Trie 树的高效自动完成：

- **前缀匹配**: 快速前缀搜索
- **频率排序**: 高频词优先
- **实时更新**: 动态添加新词
- **多语言支持**: Unicode 分词

**性能**:
- 查询时间 < 1ms
- 内存占用低
- 支持百万词汇

### 4. 搜索缓存 (Search Cache)

Redis 缓存层加速重复查询：

- **查询缓存**: 5 分钟 TTL
- **哈希键**: 基于查询参数
- **自动失效**: TTL 过期
- **手动清除**: 支持主动清除

**缓存策略**:
- 缓存命中: < 5ms
- 缓存未命中: < 50ms
- 目标命中率: > 70%

## 🏗️ 架构设计

```
┌─────────────────────────────────────────┐
│         Search Engine                   │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐  ┌──────────────┐   │
│  │   Indexer    │  │    Ranker    │   │
│  │  (Tantivy)   │  │              │   │
│  │              │  │  - Title 2x  │   │
│  │ - Full-text  │  │  - Exact 3x  │   │
│  │ - Multi-field│  │  - Recency   │   │
│  └──────────────┘  └──────────────┘   │
│                                         │
│  ┌──────────────┐  ┌──────────────┐   │
│  │ AutoComplete │  │    Cache     │   │
│  │   (Trie)     │  │   (Redis)    │   │
│  └──────────────┘  └──────────────┘   │
└─────────────────────────────────────────┘
```

## 📦 数据模型

### SearchQuery

```rust
pub struct SearchQuery {
    pub query: String,           // 查询文本
    pub limit: usize,            // 返回数量
    pub offset: usize,           // 分页偏移
    pub filters: Option<Vec<SearchFilter>>, // 过滤条件
    pub sort_by: Option<String>, // 排序字段
    pub sort_order: SortOrder,   // 排序方向
    pub fuzzy: bool,             // 模糊搜索
    pub highlight: bool,         // 高亮显示
}
```

### SearchResponse

```rust
pub struct SearchResponse {
    pub results: Vec<SearchResult>, // 搜索结果
    pub total: usize,            // 总结果数
    pub query_time_ms: u64,      // 查询时间
    pub suggestions: Vec<String>, // 搜索建议
}
```

### SearchResult

```rust
pub struct SearchResult {
    pub id: Uuid,                // 文档 ID
    pub title: String,           // 标题
    pub content: String,         // 内容片段
    pub score: f32,              // 相关性分数
    pub highlights: Vec<String>, // 高亮片段
    pub metadata: Value,         // 元数据
}
```

### Document

```rust
pub struct Document {
    pub id: Uuid,                // 文档 ID
    pub title: String,           // 标题
    pub content: String,         // 内容
    pub doc_type: String,        // 文档类型
    pub tags: Vec<String>,       // 标签
    pub metadata: Value,         // 元数据
    pub timestamp: i64,          // 时间戳
}
```

## 🚀 使用指南

### 1. 初始化搜索引擎

```rust
use pixelcore_search::{SearchEngine, SearchEngineConfig};
use std::path::PathBuf;

let config = SearchEngineConfig {
    index_path: PathBuf::from("./data/search_index"),
    redis_url: "redis://127.0.0.1:6379".to_string(),
    cache_ttl: 300, // 5 minutes
    enable_autocomplete: true,
};

let engine = SearchEngine::new(config).await?;
```

### 2. 索引文档

```rust
use pixelcore_search::query::Document;
use uuid::Uuid;

let doc = Document {
    id: Uuid::new_v4(),
    title: "Introduction to Rust".to_string(),
    content: "Rust is a systems programming language...".to_string(),
    doc_type: "article".to_string(),
    tags: vec!["rust".to_string(), "programming".to_string()],
    metadata: serde_json::json!({"author": "John Doe"}),
    timestamp: 1234567890,
};

engine.index_document(doc).await?;
```

### 3. 批量索引

```rust
let documents = vec![doc1, doc2, doc3];
engine.index_documents(documents).await?;
```

### 4. 搜索文档

```rust
use pixelcore_search::query::SearchQuery;

let query = SearchQuery {
    query: "rust programming".to_string(),
    limit: 10,
    offset: 0,
    filters: None,
    sort_by: None,
    sort_order: SortOrder::Descending,
    fuzzy: true,
    highlight: true,
};

let response = engine.search(query).await?;

for result in response.results {
    println!("{}: {} (score: {})", result.id, result.title, result.score);
}
```

### 5. 自动完成

```rust
let suggestions = engine.autocomplete("rus", 5);
// 返回: ["rust", "russian", ...]
```

### 6. 删除文档

```rust
engine.delete_document(doc_id).await?;
```

### 7. 获取统计信息

```rust
let stats = engine.stats().await?;
println!("Total documents: {}", stats.total_documents);
```

## ⚡ 性能优化

### 1. 索引优化

```rust
// 批量索引比单个索引快 10x
engine.index_documents(large_batch).await?;

// 手动提交控制
engine.commit().await?;
```

### 2. 查询优化

```rust
// 使用缓存
let query = SearchQuery {
    query: "popular query".to_string(),
    ..Default::default()
};

// 第一次查询: ~50ms
let response1 = engine.search(query.clone()).await?;

// 第二次查询（缓存命中）: ~5ms
let response2 = engine.search(query).await?;
```

### 3. 分页优化

```rust
// 使用 offset + limit 进行分页
let page1 = SearchQuery {
    query: "rust".to_string(),
    limit: 10,
    offset: 0,
    ..Default::default()
};

let page2 = SearchQuery {
    query: "rust".to_string(),
    limit: 10,
    offset: 10,
    ..Default::default()
};
```

## 📊 性能指标

### 目标指标

- **搜索响应时间**: < 50ms (无缓存)
- **缓存响应时间**: < 5ms
- **搜索准确率**: > 85%
- **索引速度**: > 1000 docs/sec
- **QPS**: > 50,000
- **缓存命中率**: > 70%

### 监控指标

```rust
// 搜索延迟
histogram!("search.latency", latency_ms);

// 缓存命中率
counter!("search.cache.hit");
counter!("search.cache.miss");

// 索引速度
gauge!("search.index.docs_per_sec", docs_per_sec);
```

## 🧪 测试

### 运行单元测试

```bash
cd crates/pixelcore-search
cargo test
```

### 运行示例程序

```bash
# 启动 Redis
docker run -d -p 6379:6379 redis

# 运行示例
cargo run --example search_demo
```

## 🔧 配置

### 环境变量

```bash
# Redis 连接
REDIS_URL=redis://127.0.0.1:6379

# 索引路径
SEARCH_INDEX_PATH=./data/search_index

# 缓存 TTL（秒）
SEARCH_CACHE_TTL=300

# 启用自动完成
SEARCH_ENABLE_AUTOCOMPLETE=true
```

## 📈 未来改进

### 短期 (1-2 周)

- [ ] 向量搜索（Embedding）
- [ ] 语义搜索
- [ ] 搜索高亮优化
- [ ] 多语言分词器

### 中期 (1-2 月)

- [ ] 搜索分析和统计
- [ ] A/B 测试框架
- [ ] 个性化搜索
- [ ] 搜索推荐

### 长期 (3-6 月)

- [ ] 深度学习排序
- [ ] 图像搜索
- [ ] 语音搜索
- [ ] 跨语言搜索

## 🔗 相关文档

- [Phase 5 计划](../PHASE5_PLAN.md)
- [AI 推荐系统](../AI_RECOMMENDATION.md)
- [缓存策略](../CACHE_STRATEGY.md)

## 📝 更新日志

### v0.1.0 (2026-03-04)

- ✅ 实现 Tantivy 全文搜索
- ✅ 实现智能排序算法
- ✅ 实现 Trie 自动完成
- ✅ 集成 Redis 缓存
- ✅ 添加单元测试
- ✅ 创建示例程序
