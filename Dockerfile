# 多阶段构建 - Rust 后端
FROM rustlang/rust:nightly-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    perl \
    && rm -rf /var/lib/apt/lists/*

# 创建工作目录
WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY app/src-tauri ./app/src-tauri

# 构建依赖（缓存层）
# 设置环境变量使用系统 OpenSSL
ENV OPENSSL_NO_VENDOR=1
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src
COPY examples ./examples

# 构建应用
ENV OPENSSL_NO_VENDOR=1
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 pixelcore

# 创建数据目录
RUN mkdir -p /data && chown pixelcore:pixelcore /data

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/pixelcore /app/pixelcore

# 切换到非 root 用户
USER pixelcore

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动应用
CMD ["./pixelcore"]
