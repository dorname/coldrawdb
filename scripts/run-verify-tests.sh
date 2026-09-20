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
# truncate 仅在 OPENLOGOS_APPEND 未设时生效（沙箱模式默认 APPEND=1）。
# 沙箱外单跑脚本时（无 APPEND）需要 truncate 防陈旧数据污染。
# 沙箱里若 truncate 失败（bwrap --ro-bind workspace），也允许继续：reporter 走 append 模式。
if [[ -z "${OPENLOGOS_APPEND:-}" ]]; then
    : > "$JSONL" 2>/dev/null || true
fi
export OPENLOGOS_APPEND=1
# OpenLogos 沙箱 (bwrap --ro-bind workspace) 让 reporter 用 CARGO_MANIFEST_DIR 解析的
# 相对路径指向沙箱副本，沙箱销毁后丢失。让 reporter 走绝对路径。
export COLDRAWDB_JSONL_PATH="$JSONL"

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

echo "[verify-pre-run] C 批 room-editor 壳层/保存态/协作 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:spec-parity-c)

echo "[verify-pre-run] D 批 IO/快捷键/主题/响应式/画布拖拽 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:spec-parity-d)

echo "[verify-pre-run] E 批 splitter 分隔条 + 表注释 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:spec-parity-e)

echo "[verify-pre-run] F 批画布性能/锚定缩放 Playwright 回归 ..."
(cd "$ROOT/frontend-rs" && "$NPM_BIN" run test:canvas-perf)

# G 批：fix-remote-github-issues-7-18 新增 e2e spec（Playwright 测试框架，
# 非手写 parity 脚本）。必须在 tests/e2e 下用其本地 playwright 实例运行。
#
# 前置：parity 各批的 `trunk build` 不带 COLDRAWDB_API_BASE，其 rlib 会污染共享
# cargo 增量缓存与 dist/（option_env! 读编译期变量，缓存复用后 serve 重建不会重烧），
# 导致页面 API 走同源 :18080 静态服务而 405。G 批用例依赖 18080 dev 服务的页面
# 直连后端 :3000，这里显式带变量重建 dist 恢复。
echo "[verify-pre-run] G 批前重建含 COLDRAWDB_API_BASE 的前端 dist ..."
(cd "$ROOT/frontend-rs" && COLDRAWDB_API_BASE="http://127.0.0.1:${COLDRAWDB_BACKEND_PORT:-3000}" trunk build)

echo "[verify-pre-run] G 批 GitHub issue 修复 Playwright 回归 ..."
(cd "$ROOT/frontend-rs/tests/e2e" && ./node_modules/.bin/playwright test \
  specs/pc-ddl-drop.spec.ts \
  specs/canvas-sides-resize.spec.ts \
  specs/appbar-truncate.spec.ts \
  specs/cr-comment-color.spec.ts)

echo "[verify-pre-run] 校验 reporter ID 与覆盖度 ..."
"$NODE_BIN" "$ROOT/scripts/validate-openlogos-ledger.mjs" --report ST-PU-20

echo "[verify-pre-run] done → $JSONL"
