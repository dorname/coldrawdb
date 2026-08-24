#!/usr/bin/env bash
# SMOKE-core-06 runner：本地脚本启停验证（logos.config.json smoke.command 入口）
# 执行 scripts/tests/test-local-scripts.sh，并把结果追加到 smoke-results.jsonl
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESULTS="${REPO_ROOT}/logos/resources/verify/smoke-results.jsonl"

started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
start_ms="$(date +%s%3N)"
status="pass"
note="SMOKE-core-06: local start-local.sh + stop-local.sh round-trip validated via scripts/tests/test-local-scripts.sh"

if ! bash "${REPO_ROOT}/scripts/tests/test-local-scripts.sh"; then
    status="fail"
fi

end_ms="$(date +%s%3N)"
duration_ms=$((end_ms - start_ms))

mkdir -p "$(dirname "$RESULTS")"
printf '{"id":"SMOKE-core-06","status":"%s","duration_ms":%s,"timestamp":"%s","scenario":"smoke","note":"%s"}\n' \
    "$status" "$duration_ms" "$started_at" "$note" >> "$RESULTS"

[[ "$status" == "pass" ]]
