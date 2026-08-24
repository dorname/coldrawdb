# 合并指令

## 变更提案
- 提案名称：harden-local-dev-cold-start
- 提案目录：logos/changes/harden-local-dev-cold-start/

## 提案内容

# 变更提案：本地启动脚本冷启动加固（harden-local-dev-cold-start）

> module: core | created: 2026-08-24

## 变更原因

`implement-unified-prototype-spec-parity` 验收期间，openlogos verify 在沙箱冷环境中暴露本地脚本的冷编译超时类问题；verify 自愈循环据此对 4 个文件生成了一组方向正确的修复（未提交、游离在工作区）。用户决定保留并通过本提案正式纳入，恢复变更可追溯性。根因：

1. `scripts/start-local.sh` 用 `cargo run --release` 启动后端，冷机首次编译期间健康检查 60 秒超时，误报启动失败。
2. `scripts/tests/test-local-scripts.sh` 设置 `COLDRAWDB_BACKEND_PORT=13000`，但后端实际绑定端口固定为 `backend/config.toml` 的 3000（该环境变量仅影响脚本侧的端口占用检查与健康检查地址），导致健康检查轮询错误端口；前端挂载点断言 `<div id="root">` 与 `frontend-rs/index.html` 实际的 `<div id="app">` 不符。

## 变更类型

代码级修复（含部署方案与 smoke 用例文档同步）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无（不涉及 S01～S06 运行时语义）
- 影响的部署方案：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`（本地启动步骤、健康检查超时默认值、`COLDRAWDB_BACKEND_PORT` 语义澄清）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`logos/resources/test/smoke/core-smoke-test-cases.md` SMOKE-core-06（健康检查 60s→120s、前端挂载点断言 `#root`→`#app`）
- 影响的脚本：`scripts/start-local.sh`、`scripts/common.sh`、`scripts/tests/test-local-scripts.sh`

## 部署影响

- 是否需要部署：否
- 部署原因：仅本地开发启停脚本与相关文档，不涉及任何部署环境
- 影响环境：无（仅本地开发工作流）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否（smoke 用例文档同步仅为 delta 更新，本提案不触发 `openlogos smoke` 执行）

## UI/UX 变更声明

```yaml
ui_impact: false            # 本次是否触及界面（GUI 项目才有意义）
design_system_mode: generated   # generated | fallback（fallback 时须填 design_system_fallback_reason）
design_system_fallback_reason: ""
pages: []                   # 每项 {id, prototype: core-NN-<slug>.html, description}
```

## 变更概述

采纳并固化 verify 自愈循环生成的 4 文件修复：

1. `scripts/start-local.sh`：启动后端前先 `cargo build --release` 预编译，随后直接执行 `./target/release/backend`；健康检查失败时检测日志中是否仍在编译并给出针对性提示。
2. `scripts/common.sh`：`COLDRAWDB_HEALTH_TIMEOUT` 默认值 60 → 120 秒。
3. `scripts/tests/test-local-scripts.sh`：测试后端端口对齐 `backend/config.toml` 固定值 3000（并 `unset COLDRAWDB_BACKEND_PORT` 避免误导），前端断言从 `root` 改为实际挂载点 `app`。
4. 部署方案文档同步上述启动步骤与超时默认值，并澄清 `COLDRAWDB_BACKEND_PORT` 仅影响脚本侧端口占用检查与健康检查地址、不改变后端实际绑定端口（固定 3000）；SMOKE-core-06 用例断言同步。


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/harden-local-dev-cold-start/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/harden-local-dev-cold-start/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(harden-local-dev-cold-start): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive harden-local-dev-cold-start`。
