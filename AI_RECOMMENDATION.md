# AI 推荐系统文档

## 📋 概述

PixelCore AI 推荐系统提供智能的个性化推荐功能，使用混合推荐算法（协同过滤 + 基于内容）来为用户推荐相关内容。

## 🎯 核心功能

### 1. 协同过滤 (Collaborative Filtering)

基于用户行为相似度的推荐算法：

- **用户-物品矩阵**: 构建用户与物品的交互矩阵
- **余弦相似度**: 计算用户之间的相似度
- **邻居推荐**: 基于相似用户的偏好进行推荐

**优点**:
- 无需物品特征信息
- 能发现意外的推荐
- 随着数据增长效果提升

**缺点**:
- 冷启动问题（新用户/新物品）
- 稀疏性问题
- 可扩展性挑战

### 2. 基于内容推荐 (Content-Based)

基于物品特征和用户偏好的推荐算法：

- **特征向量**: 物品的特征表示（10 维向量）
- **用户画像**: 基于历史交互构建用户偏好
- **相似度匹配**: 推荐与用户偏好相似的物品

**优点**:
- 不依赖其他用户数据
- 可解释性强
- 适合新物品推荐

**缺点**:
- 需要物品特征数据
- 推荐范围受限
- 可能过度专业化

### 3. 混合推荐 (Hybrid)

结合协同过滤和基于内容的优势：

1. 优先使用协同过滤
2. 不足时补充基于内容推荐
3. 综合评分和排序

## 🏗️ 架构设计

```
┌─────────────────────────────────────────┐
│      Recommendation Engine              │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐  ┌──────────────┐   │
│  │ Collaborative│  │ Content-Based│   │
│  │  Filtering   │  │              │   │
│  └──────────────┘  └──────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │    Redis Cache Layer             │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## 📦 数据模型

### RecommendationRequest

```rust
pub struct RecommendationRequest {
    pub user_id: Uuid,           // 用户 ID
    pub limit: usize,            // 推荐数量
    pub item_type: Option<String>, // 物品类型过滤
    pub exclude_items: Option<Vec<Uuid>>, // 排除物品
}
```

### RecommendationResponse

```rust
pub struct RecommendationResponse {
    pub user_id: Uuid,           // 用户 ID
    pub items: Vec<RecommendedItem>, // 推荐物品列表
    pub algorithm: String,       // 使用的算法
    pub confidence: f64,         // 置信度
}
```

### RecommendedItem

```rust
pub struct RecommendedItem {
    pub item_id: Uuid,           // 物品 ID
    pub score: f64,              // 推荐分数
    pub reason: String,          // 推荐理由
}
```

### UserInteraction

```rust
pub struct UserInteraction {
    pub user_id: Uuid,           // 用户 ID
    pub item_id: Uuid,           // 物品 ID
    pub interaction_type: InteractionType, // 交互类型
    pub rating: Option<f64>,     // 评分
    pub timestamp: i64,          // 时间戳
}

pub enum InteractionType {
    View,      // 浏览
    Click,     // 点击
    Purchase,  // 购买
    Like,      // 点赞
    Share,     // 分享
    Rating,    // 评分
}
```

## 🚀 使用指南

### 1. 初始化推荐引擎

```rust
use pixelcore_ai::RecommendationEngine;

let redis_url = "redis://127.0.0.1:6379";
let engine = RecommendationEngine::new(redis_url).await?;
```

### 2. 训练模型

```rust
use pixelcore_ai::recommendation::{UserInteraction, InteractionType};

let interactions = vec![
    UserInteraction {
        user_id: user1,
        item_id: item1,
        interaction_type: InteractionType::Purchase,
        rating: Some(5.0),
        timestamp: 1234567890,
    },
    // ... more interactions
];

engine.train(interactions).await?;
```

### 3. 获取推荐

```rust
use pixelcore_ai::recommendation::RecommendationRequest;

let request = RecommendationRequest {
    user_id: user_id,
    limit: 10,
    item_type: Some("image".to_string()),
    exclude_items: Some(vec![item1, item2]),
};

let response = engine.recommend(request).await?;

for item in response.items {
    println!("Recommend: {} (score: {})", item.item_id, item.score);
}
```

## ⚡ 性能优化

### 1. Redis 缓存

- **缓存策略**: 推荐结果缓存 1 小时
- **缓存键**: `recommendation:{user_id}`
- **失效策略**: TTL 自动过期 + 手动失效

### 2. 批量处理

```rust
// 批量训练
engine.train(large_interactions_batch).await?;

// 批量推荐
let requests = vec![req1, req2, req3];
let responses = futures::future::join_all(
    requests.into_iter().map(|req| engine.recommend(req))
).await;
```

### 3. 异步处理

所有操作都是异步的，支持高并发：

```rust
// 并发获取多个用户的推荐
let handles: Vec<_> = user_ids.into_iter()
    .map(|user_id| {
        let engine = engine.clone();
        tokio::spawn(async move {
            engine.recommend(RecommendationRequest {
                user_id,
                limit: 10,
                item_type: None,
                exclude_items: None,
            }).await
        })
    })
    .collect();

let results = futures::future::join_all(handles).await;
```

## 📊 性能指标

### 目标指标

- **推荐准确率**: > 70%
- **响应时间**: < 100ms (有缓存时 < 10ms)
- **QPS**: > 10,000
- **缓存命中率**: > 80%

### 监控指标

```rust
// 推荐延迟
histogram!("recommendation.latency", latency_ms);

// 缓存命中率
counter!("recommendation.cache.hit");
counter!("recommendation.cache.miss");

// 推荐质量
gauge!("recommendation.accuracy", accuracy);
```

## 🧪 测试

### 运行单元测试

```bash
cd crates/pixelcore-ai
cargo test
```

### 运行示例程序

```bash
# 启动 Redis
docker run -d -p 6379:6379 redis

# 运行示例
cargo run --example ai_recommendation_demo
```

## 🔧 配置

### 环境变量

```bash
# Redis 连接
REDIS_URL=redis://127.0.0.1:6379

# 缓存 TTL（秒）
RECOMMENDATION_CACHE_TTL=3600

# 推荐数量限制
RECOMMENDATION_MAX_LIMIT=100
```

## 📈 未来改进

### 短期 (1-2 周)

- [ ] 添加更多推荐算法（矩阵分解、深度学习）
- [ ] 实现 A/B 测试框架
- [ ] 添加推荐解释功能
- [ ] 优化冷启动问题

### 中期 (1-2 月)

- [ ] 实时推荐更新
- [ ] 多目标优化（点击率 + 转化率）
- [ ] 上下文感知推荐
- [ ] 推荐多样性优化

### 长期 (3-6 月)

- [ ] 深度学习推荐模型
- [ ] 强化学习优化
- [ ] 跨域推荐
- [ ] 联邦学习支持

## 🔗 相关文档

- [Phase 5 计划](../PHASE5_PLAN.md)
- [缓存策略](../CACHE_STRATEGY.md)
- [API 文档](../docs/api/ai.md)

## 📝 更新日志

### v0.1.0 (2026-03-04)

- ✅ 实现协同过滤算法
- ✅ 实现基于内容推荐
- ✅ 实现混合推荐策略
- ✅ 集成 Redis 缓存
- ✅ 添加单元测试
- ✅ 创建示例程序
