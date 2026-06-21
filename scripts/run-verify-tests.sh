#!/usr/bin/env bash
# OpenLogos verify 预跑：backend + frontend-rs 全量测试，合并写入 test-results.jsonl
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JSONL="$ROOT/logos/resources/verify/test-results.jsonl"

mkdir -p "$(dirname "$JSONL")"
: > "$JSONL"
export OPENLOGOS_APPEND=1

echo "[verify-pre-run] backend cargo test ..."
(cd "$ROOT/backend" && cargo test)

echo "[verify-pre-run] frontend-rs cargo test ..."
(cd "$ROOT/frontend-rs" && cargo test)

echo "[verify-pre-run] done → $JSONL"
