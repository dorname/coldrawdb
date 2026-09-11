# Delta — core-PV-verify-pre-run-test-cases.md（UT-PU-21 NO_COLOR 口径修订）

> 模块：core | 提案：fix-local-start-and-invite-config
> 目标主文档：`logos/resources/test/core-PV-verify-pre-run-test-cases.md`

## MODIFIED — UT-PU-21 登记表行

> merge 时替换主文档登记表中的 UT-PU-21 行。变更点：trunk 把 NO_COLOR 解析为 bool（clap 仅接受 true/false），`NO_COLOR=1` 会导致 trunk serve 直接退出；口径由 `NO_COLOR=1` 修订为 `NO_COLOR=true`，语义不变（强制无 ANSI 转义落盘日志）。

| UT-PU-21 | — | 锚点检查 `backend/src/main.rs` 与 `scripts/start-local.sh` | `init_log` 含 `with_ansi(false)` 且不含 `.pretty()`；默认 filter 含 `sqlx::query=warn`；start-local.sh 的 trunk 启动含 `NO_COLOR=true`（trunk 仅接受 bool，fix-local-start-and-invite-config 修订）且不再 `-u NO_COLOR` |

## MODIFIED — UT-PU-21 详情第 3 条

> merge 时替换主文档 UT-PU-21 详情区的第 3 条步骤：

  3. `start-local.sh` trunk 启动行含 `NO_COLOR=true` 且不含 `-u NO_COLOR`
