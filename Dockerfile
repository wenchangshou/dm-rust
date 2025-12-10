# 构建阶段
FROM rust:1.91.1-slim-trixie AS builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制 Cargo 文件用于依赖缓存
COPY Cargo.toml Cargo.lock* ./

# 创建虚拟 src 目录用于构建依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制实际源代码
COPY src ./src

# 重新构建应用
RUN touch src/main.rs && \
    cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/dm-rust /app/dm-rust

# 创建配置目录
RUN mkdir -p /app/config /app/data

# 复制配置文件模板
COPY config.json /app/config/config.json

# 暴露端口
EXPOSE 18080

# 设置环境变量
ENV RUST_LOG=info

# 启动命令
CMD ["/app/dm-rust", "--config", "/app/config/config.json", "--log-level", "info"]
