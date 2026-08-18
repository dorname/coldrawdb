#!/usr/bin/env bash
# 构建本地 MCP release 二进制；本脚本不安装、不修改任何客户端配置。
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"

"$CARGO_BIN" build --release --manifest-path "$PROJECT_ROOT/mcp-server/Cargo.toml"
echo "$PROJECT_ROOT/mcp-server/target/release/coldrawdb-mcp"
