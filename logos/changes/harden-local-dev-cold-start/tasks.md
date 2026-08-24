# 实现任务

> module: core | proposal: harden-local-dev-cold-start

## 执行约束

- 本提案正式采纳 verify 自愈循环已写入工作区的 4 文件修复；[code] 任务是对既有工作区改动的审查、确认与提交，不改语义。
- 规格文档（部署方案、smoke 用例）一律通过 delta 文件变更，不直接改主文档。
- 已知边界（不在本提案修复）：`start-local.sh` 中 `COLDRAWDB_BACKEND_PORT` 若被显式设为非 3000，健康检查将轮询错误端口——仅在文档中澄清语义，不改脚本逻辑。

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 启动步骤改为预编译后执行二进制；`COLDRAWDB_HEALTH_TIMEOUT` 默认 120s；澄清 `COLDRAWDB_BACKEND_PORT` 仅影响脚本侧检查与健康检查地址
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — SMOKE-core-06 断言同步：健康检查 120 秒内通过、前端入口 body 含 `#app` 挂载点

## [code] 代码实现

- [ ] 实现代码变更