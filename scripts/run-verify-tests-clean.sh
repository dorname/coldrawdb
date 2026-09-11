#!/usr/bin/env bash
# Wrapper: 在跑 run-verify-tests.sh 前清掉残留 trunk/Playwright 进程,
# 避免端口 4175/4173/5173 冲突导致 Playwright 启动超时。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# 按端口清理残留服务(避免 pkill -f 误杀当前脚本链)。
for port in 4175 4173 5173; do
    pids=$(ss -ltnp 2>/dev/null | grep ":$port " | grep -oP 'pid=\K[0-9]+' | sort -u || true)
    for pid in $pids; do
        kill "$pid" 2>/dev/null || true
    done
done
sleep 1

cd "$ROOT"
exec bash scripts/run-verify-tests.sh "$@"