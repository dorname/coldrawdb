# Delta: core-PV-verify-pre-run-test-cases.md

> 提案：fix-select-bg-diagram-delete-and-log-format

## ADDED — §1 新增用例表行

| UT-PU-21 | — | 锚点检查 `backend/src/main.rs` 与 `scripts/start-local.sh` | `init_log` 含 `with_ansi(false)` 且不含 `.pretty()`；默认 filter 含 `sqlx::query=warn`；start-local.sh 的 trunk 启动含 `NO_COLOR=1` 且不再 `-u NO_COLOR` |

## ADDED — 用例详情

### UT-PU-21 — 日志格式锚点（后端 ANSI/格式 + 前端启动脚本）

- **位置**：`backend/src/main.rs` 测试模块（读自身源码与 `../scripts/start-local.sh`）
- **断言**：
  1. `init_log` 含 `.with_ansi(false)`；不含 `.pretty()`
  2. 默认 EnvFilter 含 `sqlx::query=warn`（降噪；RUST_LOG 可覆盖）
  3. `start-local.sh` trunk 启动行含 `NO_COLOR=1` 且不含 `-u NO_COLOR`
