# 负载均衡配置指南

本文档介绍 PixelCore 的负载均衡配置和最佳实践。

---

## 📋 负载均衡架构

### 架构图

```
                    ┌─────────────────┐
                    │   DNS / CDN     │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Load Balancer  │
                    │    (Nginx)      │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│   Backend 1    │  │   Backend 2    │  │   Backend 3    │
│   (Active)     │  │   (Active)     │  │   (Active)     │
└────────────────┘  └────────────────┘  └────────────────┘
```

---

## 🔄 负载均衡算法

### 1. Round Robin (轮询)

**特点**:
- 简单公平
- 适合服务器性能相近的场景

**配置**:
```nginx
upstream backend_servers {
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;
}
```

### 2. Least Connections (最少连接)

**特点**:
- 动态分配
- 适合长连接场景

**配置**:
```nginx
upstream backend_servers {
    least_conn;
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;
}
```

### 3. IP Hash (IP 哈希)

**特点**:
- 会话保持
- 同一客户端总是访问同一服务器

**配置**:
```nginx
upstream backend_servers {
    ip_hash;
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;
}
```

### 4. Weighted (加权)

**特点**:
- 根据服务器性能分配
- 适合服务器性能不同的场景

**配置**:
```nginx
upstream backend_servers {
    server backend-1:8080 weight=3;  # 高性能服务器
    server backend-2:8080 weight=2;  # 中等性能
    server backend-3:8080 weight=1;  # 低性能服务器
}
```

---

## 🏥 健康检查

### Nginx 健康检查

```nginx
upstream backend_servers {
    server backend-1:8080 max_fails=3 fail_timeout=30s;
    server backend-2:8080 max_fails=3 fail_timeout=30s;
    server backend-3:8080 max_fails=3 fail_timeout=30s;

    # 备用服务器
    server backend-backup:8080 backup;
}
```

**参数说明**:
- `max_fails`: 最大失败次数
- `fail_timeout`: 失败超时时间
- `backup`: 备用服务器，仅在主服务器全部失败时使用

### 主动健康检查

```nginx
location /health {
    access_log off;
    proxy_pass http://backend_servers/health;
    proxy_http_version 1.1;
    proxy_set_header Connection "";

    # 健康检查超时
    proxy_connect_timeout 2s;
    proxy_send_timeout 2s;
    proxy_read_timeout 2s;
}
```

---

## 🔌 连接管理

### Keepalive 连接

```nginx
upstream backend_servers {
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;

    # Keepalive 配置
    keepalive 32;                    # 保持 32 个空闲连接
    keepalive_timeout 60s;           # 连接超时时间
    keepalive_requests 100;          # 每个连接最多处理 100 个请求
}

server {
    location / {
        proxy_pass http://backend_servers;
        proxy_http_version 1.1;
        proxy_set_header Connection "";  # 清除 Connection 头
    }
}
```

### 连接限制

```nginx
# 限制每个 IP 的并发连接数
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

server {
    location / {
        limit_conn conn_limit 10;  # 每个 IP 最多 10 个并发连接
        proxy_pass http://backend_servers;
    }
}
```

---

## 🚦 速率限制

### 请求速率限制

```nginx
# 定义速率限制区域
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=100r/s;
limit_req_zone $binary_remote_addr zone=login_limit:10m rate=5r/m;

server {
    # API 端点限制
    location /api/ {
        limit_req zone=api_limit burst=20 nodelay;
        proxy_pass http://backend_servers;
    }

    # 登录端点限制
    location /api/auth/login {
        limit_req zone=login_limit burst=2 nodelay;
        proxy_pass http://backend_servers;
    }
}
```

**参数说明**:
- `rate`: 平均速率
- `burst`: 突发请求数
- `nodelay`: 不延迟处理突发请求

---

## 🔒 会话保持

### 1. IP Hash

```nginx
upstream backend_servers {
    ip_hash;
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;
}
```

### 2. Cookie-based

```nginx
upstream backend_servers {
    server backend-1:8080;
    server backend-2:8080;
    server backend-3:8080;

    sticky cookie srv_id expires=1h domain=.example.com path=/;
}
```

### 3. Redis Session Store

```rust
// 使用 Redis 存储会话
async fn store_session(
    cache: &mut CacheManager,
    session_id: &str,
    session_data: &SessionData
) -> Result<()> {
    let key = format!("session:{}", session_id);
    cache.set_with_ttl(&key, session_data, Duration::from_secs(3600)).await?;
    Ok(())
}

async fn get_session(
    cache: &mut CacheManager,
    session_id: &str
) -> Result<SessionData> {
    let key = format!("session:{}", session_id);
    cache.get(&key).await
}
```

---

## 📊 监控和日志

### 访问日志

```nginx
log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                '$status $body_bytes_sent "$http_referer" '
                '"$http_user_agent" "$http_x_forwarded_for" '
                'rt=$request_time uct="$upstream_connect_time" '
                'uht="$upstream_header_time" urt="$upstream_response_time"';

access_log /var/log/nginx/access.log main;
```

### 状态监控

```nginx
server {
    listen 8080;
    server_name localhost;

    location /nginx_status {
        stub_status on;
        access_log off;
        allow 127.0.0.1;
        deny all;
    }
}
```

### Prometheus 指标

```nginx
# 安装 nginx-prometheus-exporter
# https://github.com/nginxinc/nginx-prometheus-exporter

# 暴露指标
location /metrics {
    allow 10.0.0.0/8;
    deny all;
    proxy_pass http://localhost:9113/metrics;
}
```

---

## 🔧 性能优化

### 1. Worker 进程配置

```nginx
# 自动设置为 CPU 核心数
worker_processes auto;

# 每个 worker 的最大连接数
events {
    worker_connections 1024;
    use epoll;  # Linux 上使用 epoll
}
```

### 2. 缓冲区配置

```nginx
http {
    # 客户端请求体缓冲
    client_body_buffer_size 128k;
    client_max_body_size 50m;

    # 客户端请求头缓冲
    client_header_buffer_size 1k;
    large_client_header_buffers 4 8k;

    # 代理缓冲
    proxy_buffering on;
    proxy_buffer_size 4k;
    proxy_buffers 8 4k;
    proxy_busy_buffers_size 8k;
}
```

### 3. 超时配置

```nginx
http {
    # 客户端超时
    client_body_timeout 12s;
    client_header_timeout 12s;
    send_timeout 10s;

    # Keepalive 超时
    keepalive_timeout 65s;
    keepalive_requests 100;

    # 代理超时
    proxy_connect_timeout 5s;
    proxy_send_timeout 60s;
    proxy_read_timeout 60s;
}
```

### 4. 文件缓存

```nginx
http {
    # 打开文件缓存
    open_file_cache max=1000 inactive=20s;
    open_file_cache_valid 30s;
    open_file_cache_min_uses 2;
    open_file_cache_errors on;
}
```

---

## 🌐 CDN 集成

### CloudFlare 配置

```nginx
# 获取真实 IP
set_real_ip_from 173.245.48.0/20;
set_real_ip_from 103.21.244.0/22;
set_real_ip_from 103.22.200.0/22;
# ... 更多 CloudFlare IP 范围

real_ip_header CF-Connecting-IP;
real_ip_recursive on;

# 缓存控制
location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
    add_header CDN-Cache-Control "public, max-age=31536000";
}
```

### AWS CloudFront 配置

```nginx
# 获取真实 IP
set_real_ip_from 0.0.0.0/0;
real_ip_header X-Forwarded-For;
real_ip_recursive on;

# 自定义头
add_header X-Cache-Status $upstream_cache_status;
add_header X-Served-By $hostname;
```

---

## 🔐 安全配置

### 1. SSL/TLS

```nginx
# SSL 配置
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers HIGH:!aNULL:!MD5;
ssl_prefer_server_ciphers on;

# SSL 会话缓存
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;

# OCSP Stapling
ssl_stapling on;
ssl_stapling_verify on;
ssl_trusted_certificate /etc/nginx/ssl/ca-cert.pem;
```

### 2. 安全头

```nginx
# 安全头
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "no-referrer-when-downgrade" always;
add_header Content-Security-Policy "default-src 'self'" always;
```

### 3. DDoS 防护

```nginx
# 连接限制
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;
limit_conn conn_limit 10;

# 请求速率限制
limit_req_zone $binary_remote_addr zone=req_limit:10m rate=10r/s;
limit_req zone=req_limit burst=20 nodelay;

# 慢速攻击防护
client_body_timeout 10s;
client_header_timeout 10s;
send_timeout 10s;
```

---

## 🐛 故障排查

### 1. 检查 Upstream 状态

```bash
# 查看 Nginx 状态
curl http://localhost:8080/nginx_status

# 查看错误日志
tail -f /var/log/nginx/error.log

# 测试配置
nginx -t
```

### 2. 常见问题

**502 Bad Gateway**:
- 检查后端服务是否运行
- 检查防火墙规则
- 检查超时配置

**504 Gateway Timeout**:
- 增加超时时间
- 检查后端性能
- 优化数据库查询

**499 Client Closed Request**:
- 客户端提前关闭连接
- 检查响应时间
- 优化处理速度

---

## 📈 性能测试

### 使用 wrk

```bash
# 基准测试
wrk -t12 -c400 -d30s http://localhost/api/health

# 带脚本测试
wrk -t12 -c400 -d30s -s script.lua http://localhost/api/
```

### 使用 ab

```bash
# Apache Bench
ab -n 10000 -c 100 http://localhost/api/health
```

### 使用 k6

```javascript
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '1m', target: 500 },
    { duration: '30s', target: 0 },
  ],
};

export default function () {
  const res = http.get('http://localhost/api/health');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

---

## 📚 参考资料

- [Nginx 官方文档](https://nginx.org/en/docs/)
- [Nginx 负载均衡](https://docs.nginx.com/nginx/admin-guide/load-balancer/)
- [性能优化指南](https://www.nginx.com/blog/tuning-nginx/)

---

**负载均衡愉快！** 🚀
