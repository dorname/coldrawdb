# syntax=docker/dockerfile:1
# feat-docker-compose-deploy（core-01 部署方案 §4.1）：Rust 全栈单镜像。
# 旧 JS 栈 Dockerfile（npm + nginx）已废弃——仓库根无 package.json，必然构建失败。

# ---------- Stage 1: WASM 前端构建（trunk 入口 frontend-rs/index.html） ----------
FROM rust:1.89-bookworm AS wasm-build
WORKDIR /app
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
COPY frontend-rs/Cargo.toml frontend-rs/Cargo.lock ./frontend-rs/
COPY frontend-rs/index.html frontend-rs/Trunk.toml ./frontend-rs/
COPY frontend-rs/src ./frontend-rs/src
# index.html data-wasm-opt="0"（-O0）；trunk 0.21.14 无 --no-wasm-opt flag，wasm-opt 由 trunk 自动下载
RUN cd frontend-rs && trunk build --release

# ---------- Stage 2: 后端构建 ----------
FROM rust:1.89-bookworm AS api-build
WORKDIR /app
COPY backend ./backend
RUN cargo build --release --manifest-path backend/Cargo.toml

# ---------- Stage 3: 运行时 ----------
FROM debian:bookworm-slim
# sqlx 静态链接 SQLite，无需 libsqlite3-0；curl 供 compose healthcheck；Retries 抗 CDN 抖动
# 构建环境可能注入 HTTP(S)_PROXY（本地代理对部分 .deb 返回 502），apt 直连绕过
RUN unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy \
    && apt-get update -o Acquire::Retries=5 \
    && apt-get install -y --no-install-recommends -o Acquire::Retries=5 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
# 后端按 cwd 相对路径读取 config.toml / init.sql / migrations/，固定 WORKDIR
WORKDIR /app
COPY --from=api-build /app/backend/target/release/backend /usr/local/bin/backend
COPY backend/config.toml /app/config.toml
COPY backend/init.sql /app/init.sql
COPY backend/migrations /app/migrations
COPY --from=wasm-build /app/frontend-rs/dist /var/www/
RUN mkdir -p /data
ENV COLDRAWDB_BIND_ADDR="0.0.0.0:3000" \
    COLDRAWDB_STATIC_DIR="/var/www" \
    COLDRAWDB_DB_URL="sqlite:///data/coldrawdb.db?mode=rwc"
EXPOSE 3000
CMD ["backend"]
