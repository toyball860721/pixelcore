# 缓存策略配置

本文档定义了 PixelCore 的缓存策略和最佳实践。

---

## 📋 缓存层次

### 1. 浏览器缓存 (Browser Cache)

**静态资源**:
- CSS/JS/Images: 1 年
- Fonts: 1 年
- Cache-Control: `public, immutable`

**HTML 文件**:
- 缓存时间: 1 小时
- Cache-Control: `public, must-revalidate`

### 2. CDN 缓存 (CDN Cache)

**静态资源**:
- 缓存时间: 7 天
- 自动失效: 源站更新时

**API 响应**:
- GET 请求: 5 分钟
- 其他请求: 不缓存

### 3. Nginx 缓存 (Proxy Cache)

**API 缓存**:
- 缓存路径: `/var/cache/nginx/api`
- 缓存大小: 1GB
- 缓存时间: 5 分钟
- 失效时间: 60 分钟

**静态文件缓存**:
- 缓存路径: `/var/cache/nginx/static`
- 缓存大小: 5GB
- 缓存时间: 7 天
- 失效时间: 7 天

### 4. Redis 缓存 (Application Cache)

**数据缓存**:
- 用户会话: 24 小时
- 用户资料: 1 小时
- Agent 列表: 5 分钟
- 交易记录: 10 分钟

**计数器**:
- API 调用计数: 1 小时
- 速率限制: 1 分钟

---

## 🔑 缓存键设计

### 命名规范

```
{namespace}:{entity}:{id}:{version}
```

### 示例

```
user:profile:123:v1
agent:list:page:1:v1
transaction:detail:456:v1
api:rate_limit:192.168.1.1:v1
```

### 命名空间

- `user:` - 用户相关数据
- `agent:` - Agent 相关数据
- `transaction:` - 交易相关数据
- `api:` - API 相关数据
- `session:` - 会话数据
- `cache:` - 通用缓存

---

## ⏱️ TTL 配置

### 短期缓存 (< 5 分钟)

适用于:
- 实时数据
- 频繁变化的数据
- 搜索结果

```rust
cache.set_with_ttl(key, &data, Duration::from_secs(60)).await?;
```

### 中期缓存 (5-60 分钟)

适用于:
- 用户资料
- Agent 列表
- 配置数据

```rust
cache.set_with_ttl(key, &data, Duration::from_secs(300)).await?;
```

### 长期缓存 (> 1 小时)

适用于:
- 静态内容
- 历史数据
- 统计数据

```rust
cache.set_with_ttl(key, &data, Duration::from_secs(3600)).await?;
```

---

## 🔄 缓存更新策略

### 1. Cache-Aside (旁路缓存)

```rust
async fn get_user_profile(cache: &mut CacheManager, user_id: u64) -> Result<UserProfile> {
    let key = format!("user:profile:{}", user_id);

    cache.get_or_set(&key, || async {
        // Fetch from database
        database.get_user(user_id).await
    }).await
}
```

### 2. Write-Through (写穿)

```rust
async fn update_user_profile(
    cache: &mut CacheManager,
    db: &Database,
    user_id: u64,
    profile: &UserProfile
) -> Result<()> {
    // Update database
    db.update_user(user_id, profile).await?;

    // Update cache
    let key = format!("user:profile:{}", user_id);
    cache.set(&key, profile).await?;

    Ok(())
}
```

### 3. Write-Behind (写回)

```rust
async fn update_user_profile_async(
    cache: &mut CacheManager,
    queue: &mut Queue,
    user_id: u64,
    profile: &UserProfile
) -> Result<()> {
    // Update cache immediately
    let key = format!("user:profile:{}", user_id);
    cache.set(&key, profile).await?;

    // Queue database update
    queue.push(UpdateTask {
        user_id,
        profile: profile.clone(),
    }).await?;

    Ok(())
}
```

### 4. Refresh-Ahead (预刷新)

```rust
async fn get_with_refresh(
    cache: &mut CacheManager,
    key: &str,
    fetch_fn: impl Fn() -> Future<Output = Result<T>>
) -> Result<T> {
    let ttl = cache.ttl(key).await?;

    // If TTL < 10% of original, refresh in background
    if ttl > 0 && ttl < 60 {
        tokio::spawn(async move {
            let value = fetch_fn().await?;
            cache.set(key, &value).await?;
        });
    }

    cache.get(key).await
}
```

---

## 🚫 缓存失效策略

### 1. 基于时间的失效 (TTL)

```rust
// 设置 TTL
cache.set_with_ttl(key, &data, Duration::from_secs(300)).await?;

// 更新 TTL
cache.expire(key, Duration::from_secs(600)).await?;
```

### 2. 主动失效

```rust
// 删除单个键
cache.delete(&format!("user:profile:{}", user_id)).await?;

// 删除模式匹配的键
cache.clear_pattern("user:profile:*").await?;
```

### 3. 事件驱动失效

```rust
// 监听数据库更新事件
event_bus.subscribe("user.updated", |event| async move {
    let user_id = event.user_id;
    cache.delete(&format!("user:profile:{}", user_id)).await?;
});
```

---

## 📊 缓存监控

### 关键指标

1. **命中率 (Hit Rate)**
   - 目标: > 80%
   - 计算: hits / (hits + misses)

2. **响应时间 (Response Time)**
   - 缓存命中: < 10ms
   - 缓存未命中: < 100ms

3. **内存使用 (Memory Usage)**
   - Redis: < 80% 最大内存
   - Nginx: < 80% 缓存大小

4. **驱逐率 (Eviction Rate)**
   - 目标: < 5%
   - 高驱逐率表示缓存空间不足

### 监控命令

```bash
# Redis 信息
redis-cli INFO stats

# Nginx 缓存状态
curl -s http://localhost/api/health | grep X-Cache-Status

# 缓存大小
du -sh /var/cache/nginx/*
```

---

## 🔧 缓存配置

### Redis 配置

```conf
# 最大内存
maxmemory 2gb

# 驱逐策略
maxmemory-policy allkeys-lru

# 持久化
save 900 1
save 300 10
save 60 10000

# AOF
appendonly yes
appendfsync everysec
```

### Nginx 缓存配置

```nginx
# API 缓存
proxy_cache_path /var/cache/nginx/api
    levels=1:2
    keys_zone=api_cache:10m
    max_size=1g
    inactive=60m
    use_temp_path=off;

# 缓存键
proxy_cache_key "$scheme$request_method$host$request_uri";

# 缓存有效期
proxy_cache_valid 200 5m;
proxy_cache_valid 404 1m;

# 缓存使用策略
proxy_cache_use_stale error timeout updating;
proxy_cache_background_update on;
proxy_cache_lock on;
```

---

## 🎯 最佳实践

### 1. 缓存什么

✅ **应该缓存**:
- 读多写少的数据
- 计算密集的结果
- 外部 API 响应
- 静态内容

❌ **不应该缓存**:
- 敏感数据 (密码、令牌)
- 实时数据 (股票价格)
- 用户特定的私密数据
- 频繁变化的数据

### 2. 缓存键设计

✅ **好的设计**:
```
user:profile:123:v1
agent:list:category:tech:page:1:v1
```

❌ **不好的设计**:
```
user123
agents_tech_1
```

### 3. TTL 设置

✅ **合理的 TTL**:
- 根据数据变化频率设置
- 使用不同的 TTL 层次
- 考虑业务需求

❌ **不合理的 TTL**:
- 所有数据使用相同 TTL
- TTL 过长导致数据陈旧
- TTL 过短导致缓存无效

### 4. 缓存预热

```rust
async fn warmup_cache(cache: &mut CacheManager, db: &Database) -> Result<()> {
    // 预加载热门数据
    let popular_agents = db.get_popular_agents(100).await?;
    for agent in popular_agents {
        let key = format!("agent:detail:{}", agent.id);
        cache.set(&key, &agent).await?;
    }

    Ok(())
}
```

### 5. 缓存降级

```rust
async fn get_with_fallback(
    cache: &mut CacheManager,
    db: &Database,
    key: &str
) -> Result<Data> {
    // 尝试从缓存获取
    match cache.get(key).await {
        Ok(data) => Ok(data),
        Err(_) => {
            // 缓存失败，直接从数据库获取
            db.get(key).await
        }
    }
}
```

---

## 🐛 常见问题

### 1. 缓存穿透

**问题**: 查询不存在的数据，导致每次都查询数据库

**解决方案**:
```rust
// 缓存空值
if let None = db.get(key).await? {
    cache.set_with_ttl(key, &Option::<Data>::None, Duration::from_secs(60)).await?;
}
```

### 2. 缓存击穿

**问题**: 热点数据过期，大量请求同时查询数据库

**解决方案**:
```rust
// 使用互斥锁
let lock_key = format!("lock:{}", key);
if cache.set_nx(&lock_key, "1", Duration::from_secs(10)).await? {
    let data = db.get(key).await?;
    cache.set(key, &data).await?;
    cache.delete(&lock_key).await?;
}
```

### 3. 缓存雪崩

**问题**: 大量缓存同时过期，导致数据库压力激增

**解决方案**:
```rust
// 随机化 TTL
let base_ttl = 300;
let random_offset = rand::random::<u64>() % 60;
let ttl = Duration::from_secs(base_ttl + random_offset);
cache.set_with_ttl(key, &data, ttl).await?;
```

---

## 📚 参考资料

- [Redis 官方文档](https://redis.io/documentation)
- [Nginx 缓存指南](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_cache)
- [缓存最佳实践](https://aws.amazon.com/caching/best-practices/)

---

**缓存愉快！** 🚀
