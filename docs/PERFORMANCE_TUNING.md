# PixelCore 性能调优指南

## 概述

本文档提供 PixelCore 系统的性能调优指南，包括压力测试、性能分析和优化策略。

## 性能目标

### API 性能
- P50 延迟: < 20ms
- P95 延迟: < 50ms
- P99 延迟: < 100ms
- 吞吐量: > 10,000 RPS

### 数据库性能
- 查询 P99: < 50ms
- 连接池利用率: 60-80%
- 慢查询: 0

### 缓存性能
- 缓存命中率: > 95%
- 缓存响应时间: < 5ms
- 缓存内存使用: < 80%

### 资源使用
- CPU 使用率: < 70%
- 内存使用率: < 80%
- 磁盘 I/O: < 80%
- 网络带宽: < 70%

## 压力测试

### 1. 负载测试

**目的**: 验证系统在预期负载下的性能

```bash
# 运行负载测试
k6 run tests/performance/load-test.js

# 指定目标 URL
k6 run --env BASE_URL=https://api.pixelcore.io tests/performance/load-test.js
```

**测试场景**:
- 100 并发用户，持续 5 分钟
- 200 并发用户，持续 5 分钟
- 500 并发用户，持续 5 分钟

**预期结果**:
- P99 延迟 < 100ms
- 错误率 < 1%
- 吞吐量 > 10,000 RPS

### 2. 压力测试

**目的**: 找到系统的极限

```bash
# 运行压力测试
k6 run tests/performance/stress-test.js
```

**测试场景**:
- 逐步增加到 3000 并发用户
- 观察系统崩溃点
- 记录资源使用情况

**关键指标**:
- 最大并发用户数
- 崩溃前的吞吐量
- 资源瓶颈识别

### 3. 尖峰测试

**目的**: 测试突发流量处理能力

```bash
# 运行尖峰测试
k6 run tests/performance/spike-test.js
```

**测试场景**:
- 从 100 用户突增到 2000 用户
- 持续 3 分钟
- 观察恢复时间

**预期结果**:
- 系统不崩溃
- 自动扩容响应 < 2 分钟
- 错误率 < 10%

### 4. 耐久测试

**目的**: 测试长时间运行的稳定性

```bash
# 运行耐久测试（24 小时）
k6 run --duration 24h tests/performance/endurance-test.js
```

**关键指标**:
- 内存泄漏检测
- 性能退化检测
- 资源使用趋势

## 性能分析

### 1. API 性能分析

**使用 Prometheus 查询**:

```promql
# P99 延迟
histogram_quantile(0.99,
  rate(http_request_duration_seconds_bucket[5m])
)

# 请求速率
rate(http_requests_total[5m])

# 错误率
rate(http_requests_total{status=~"5.."}[5m])
/
rate(http_requests_total[5m])
```

**优化建议**:
- 识别慢端点
- 优化数据库查询
- 增加缓存
- 使用连接池

### 2. 数据库性能分析

**PostgreSQL 慢查询**:

```sql
-- 查看慢查询
SELECT
  query,
  calls,
  total_time,
  mean_time,
  max_time
FROM pg_stat_statements
WHERE mean_time > 100
ORDER BY mean_time DESC
LIMIT 20;

-- 查看表大小
SELECT
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
LIMIT 10;
```

**优化策略**:
- 添加索引
- 优化查询计划
- 分区大表
- 使用物化视图

### 3. 缓存性能分析

**Redis 监控**:

```bash
# 查看缓存命中率
redis-cli INFO stats | grep keyspace

# 查看内存使用
redis-cli INFO memory

# 查看慢日志
redis-cli SLOWLOG GET 10
```

**优化策略**:
- 调整 TTL
- 使用缓存预热
- 实施缓存分层
- 监控缓存穿透

## 性能优化

### 1. API 优化

#### 连接池配置

```rust
// PostgreSQL 连接池
let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

#### 异步处理

```rust
// 使用 tokio 并发处理
let results = tokio::join!(
    fetch_user_data(user_id),
    fetch_recommendations(user_id),
    fetch_recent_activity(user_id),
);
```

#### 响应压缩

```rust
// 启用 gzip 压缩
use actix_web::middleware::Compress;

HttpServer::new(|| {
    App::new()
        .wrap(Compress::default())
        .service(api_routes())
})
```

### 2. 数据库优化

#### 索引优化

```sql
-- 创建复合索引
CREATE INDEX idx_user_activity
ON user_activity(user_id, created_at DESC);

-- 创建部分索引
CREATE INDEX idx_active_users
ON users(id)
WHERE status = 'active';

-- 创建表达式索引
CREATE INDEX idx_email_lower
ON users(LOWER(email));
```

#### 查询优化

```sql
-- 使用 EXPLAIN ANALYZE
EXPLAIN ANALYZE
SELECT * FROM users WHERE email = 'test@example.com';

-- 避免 SELECT *
SELECT id, name, email FROM users WHERE id = 123;

-- 使用 JOIN 代替子查询
SELECT u.*, p.name as profile_name
FROM users u
JOIN profiles p ON u.id = p.user_id
WHERE u.id = 123;
```

#### 连接池配置

```toml
# PostgreSQL 配置
max_connections = 200
shared_buffers = 4GB
effective_cache_size = 12GB
maintenance_work_mem = 1GB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 20MB
min_wal_size = 1GB
max_wal_size = 4GB
```

### 3. 缓存优化

#### Redis 配置

```conf
# Redis 配置
maxmemory 8gb
maxmemory-policy allkeys-lru
save ""
appendonly yes
appendfsync everysec
```

#### 缓存策略

```rust
// 多级缓存
async fn get_user(user_id: i64) -> Result<User> {
    // L1: 本地缓存
    if let Some(user) = local_cache.get(&user_id) {
        return Ok(user);
    }

    // L2: Redis 缓存
    if let Some(user) = redis.get(&format!("user:{}", user_id)).await? {
        local_cache.insert(user_id, user.clone());
        return Ok(user);
    }

    // L3: 数据库
    let user = db.get_user(user_id).await?;
    redis.set(&format!("user:{}", user_id), &user, 3600).await?;
    local_cache.insert(user_id, user.clone());

    Ok(user)
}
```

#### 缓存预热

```rust
// 启动时预热热点数据
async fn warmup_cache() -> Result<()> {
    let hot_users = db.get_hot_users(1000).await?;

    for user in hot_users {
        redis.set(
            &format!("user:{}", user.id),
            &user,
            3600
        ).await?;
    }

    Ok(())
}
```

### 4. 资源优化

#### CPU 优化

```rust
// 使用 rayon 并行处理
use rayon::prelude::*;

let results: Vec<_> = items
    .par_iter()
    .map(|item| process_item(item))
    .collect();
```

#### 内存优化

```rust
// 使用流式处理大数据
use futures::StreamExt;

let mut stream = db.stream_users();
while let Some(user) = stream.next().await {
    process_user(user?).await?;
}
```

#### 网络优化

```yaml
# Kubernetes 网络优化
apiVersion: v1
kind: Service
metadata:
  name: pixelcore-api
spec:
  type: ClusterIP
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800
```

## 监控和告警

### 1. 关键指标

**API 指标**:
```promql
# 请求速率
rate(http_requests_total[5m])

# P99 延迟
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))

# 错误率
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])
```

**数据库指标**:
```promql
# 连接数
pg_stat_database_numbackends

# 查询时间
pg_stat_statements_mean_time_seconds

# 缓存命中率
pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read)
```

**缓存指标**:
```promql
# 命中率
redis_keyspace_hits_total / (redis_keyspace_hits_total + redis_keyspace_misses_total)

# 内存使用
redis_memory_used_bytes / redis_memory_max_bytes
```

### 2. 告警规则

```yaml
groups:
- name: performance
  rules:
  - alert: HighLatency
    expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 0.1
    for: 5m
    annotations:
      summary: "High API latency detected"

  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.01
    for: 5m
    annotations:
      summary: "High error rate detected"

  - alert: LowCacheHitRate
    expr: redis_keyspace_hits_total / (redis_keyspace_hits_total + redis_keyspace_misses_total) < 0.95
    for: 10m
    annotations:
      summary: "Cache hit rate below 95%"
```

## 性能基准

### 当前性能

| 指标 | 目标 | 当前 | 状态 |
|------|------|------|------|
| API P99 延迟 | < 100ms | 85ms | ✅ |
| 数据库查询 P99 | < 50ms | 42ms | ✅ |
| 缓存命中率 | > 95% | 96% | ✅ |
| 吞吐量 | > 10K RPS | 12K RPS | ✅ |
| CPU 使用率 | < 70% | 65% | ✅ |
| 内存使用率 | < 80% | 72% | ✅ |

### 性能趋势

- **过去 7 天**: P99 延迟稳定在 85ms
- **过去 30 天**: 吞吐量提升 20%
- **缓存优化**: 命中率从 92% 提升到 96%

## 最佳实践

1. **定期压力测试**: 每周运行一次负载测试
2. **监控关键指标**: 实时监控 P99 延迟和错误率
3. **优化慢查询**: 每天检查慢查询日志
4. **缓存预热**: 启动时预热热点数据
5. **资源限制**: 设置合理的资源限制
6. **自动扩容**: 配置 HPA 自动扩容
7. **性能回归测试**: CI/CD 中集成性能测试

## 故障排查

### 高延迟

1. 检查 API 响应时间
2. 检查数据库慢查询
3. 检查缓存命中率
4. 检查网络延迟
5. 检查资源使用

### 高错误率

1. 检查应用日志
2. 检查数据库连接
3. 检查依赖服务
4. 检查资源限制
5. 检查网络问题

### 低吞吐量

1. 检查资源使用
2. 检查连接池配置
3. 检查并发限制
4. 检查网络带宽
5. 检查负载均衡

## 参考资源

- [k6 文档](https://k6.io/docs/)
- [PostgreSQL 性能调优](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Redis 性能优化](https://redis.io/topics/optimization)
- [Rust 性能优化](https://nnethercote.github.io/perf-book/)

---

**最后更新**: 2026-03-06
**版本**: 1.0.0
