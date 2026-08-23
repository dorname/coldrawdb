#!/usr/bin/env bash
# OpenLogos verify 预跑：原子生成 backend + frontend-rs + 单文件原型的完整测试账本
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JSONL="$ROOT/logos/resources/verify/test-results.jsonl"
VERIFY_TMP_BASE="${TMPDIR:-/tmp}"
VERIFY_TMP_DIR="$(mktemp -d "$VERIFY_TMP_BASE/coldrawdb-verify.XXXXXX")"
BACKUP="$VERIFY_TMP_DIR/test-results.before.jsonl"
HAD_RESULT=0
CARGO_BIN="${CARGO_BIN:-cargo}"
NPM_BIN="${NPM_BIN:-npm}"
NODE_BIN="${NODE_BIN:-node}"

if [[ -f "$JSONL" ]]; then
  cp "$JSONL" "$BACKUP"
  HAD_RESULT=1
fi

restore_or_cleanup() {
  local status=$?
  trap - EXIT
  if [[ $status -ne 0 ]]; then
    if [[ $HAD_RESULT -eq 1 ]]; then
      cp "$BACKUP" "$JSONL"
    else
      rm -f "$JSONL"
    fi
    echo "[verify-pre-run] 失败，已恢复运行前测试账本。" >&2
  fi
  rm -f "$BACKUP"
  rmdir "$VERIFY_TMP_DIR" 2>/dev/null || true
  exit "$status"
}
trap restore_or_cleanup EXIT

mkdir -p "$(dirname "$JSONL")"
: > "$JSONL"
export OPENLOGOS_APPEND=1

echo "[verify-pre-run] backend cargo test ..."
(cd "$ROOT/backend" && "$CARGO_BIN" test)

echo "[verify-pre-run] frontend-rs cargo test ..."
(cd "$ROOT/frontend-rs" && "$CARGO_BIN" test)

echo "[verify-pre-run] MCP cargo test ..."
(cd "$ROOT/mcp-server" && "$CARGO_BIN" test)

echo "[verify-pre-run] 解析 Playwright 浏览器 ..."
eval "$("$NODE_BIN" "$ROOT/frontend-rs/scripts/resolve-playwright-browsers.mjs" --export-env)"
echo "[verify-pre-run] PLAYWRIGHT_BROWSERS_PATH=${PLAYWRIGHT_BROWSERS_PATH:-unset} HEADLESS_SHELL=${PLAYWRIGHT_CHROMIUM_USE_HEADLESS_SHELL:-default}"

echo "[verify-pre-run] 单文件原型 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:unified-prototype)

echo "[verify-pre-run] A 批生产前端 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:spec-parity-a)

echo "[verify-pre-run] B 批房间创建 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:spec-parity-b)

echo "[verify-pre-run] 校验 reporter ID 与覆盖度 ..."
"$NODE_BIN" "$ROOT/scripts/validate-openlogos-ledger.mjs" --report ST-PU-20

echo "[verify-pre-run] done → $JSONL"
