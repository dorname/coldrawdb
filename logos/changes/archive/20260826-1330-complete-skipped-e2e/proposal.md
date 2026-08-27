# 变更提案：complete-skipped-e2e

> module: core | created: 2026-08-26
> 前置：core 模块已 launched（`openlogos verify` 267 用例中 211 pass / 0 fail / **56 skip**）

## 变更原因

`openlogos verify` 报告显示当前状态：267 用例中 211 pass / 0 fail / 56 skip。**所有 skip 均为浏览器端 e2e harness 待接入**：

- `ST-FE-S03-01`～`ST-FE-S03-05`（5）— S03 鉴权生产接入的浏览器回归
- `ST-FE-S04-01`～`ST-FE-S04-06`（6）— S04 房间/邀请生产接入的浏览器回归
- `ST-FE-S05-01`～`ST-FE-S05-06`（6）— S05 WS/OT/presence 生产接入的浏览器回归
- `ST-FE-V2-01`～`ST-FE-V2-04`（4）— V2 全链路回归
- `ST-FE-PROTO-01`～`ST-FE-PROTO-08`（8）— 统一原型视觉对齐
- `ST-CR-01` / `ST-MM-01`～`ST-MM-03` / `ST-PC-01` / `ST-SP-01` / `ST-UI-05`（9）— 画布/快捷键/导入导出/侧栏/模态回归（多为 e2e harness 占位）
- `UT-S01-02` 等（11）— 单元测试由 e2e harness 一并覆盖

`implementation-checklist.md` §7.5、§7.6 已标注 "**已由 reporter 标记为 e2e harness 待接入**"。本提案把这些 skip 收口。

## 变更类型

测试基础设施变更（新增 playwright e2e harness + 标记 56 个 skip 为 pass）。

## 变更范围

- 影响的文档：
  - `logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` 新增 §6 e2e harness 部署
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md` 等 S03~S05 设计 doc 标注 e2e 入口
- 影响的代码：
  - `frontend-rs/tests/e2e/` 新增 playwright harness
  - `scripts/run-e2e.sh`（参照现有 `scripts/run-verify-tests.sh`、`scripts/smoke-local-scripts.sh`）
  - `scripts/run-verify-tests.sh` 增加 e2e 阶段
- 影响的测试用例：56 个 skip 用例新增 harness 入口
- 影响的编排测试：无
- 影响的 smoke：新增 `SMOKE-core-07-e2e-harness` 步骤
- 影响的部署方案：e2e harness 仅在 CI / 本地 verify 启用，不进生产镜像
- 影响的 API：无
- 影响的 DB 表：无

## 部署影响

- 是否需要部署：否
- 部署原因：无
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（仅测试基础设施）
- 是否需要 smoke：否（harness 行为由 verify 覆盖）

## 变更概述

### 交付物

1. **Playwright harness**
   - 在 `frontend-rs/tests/e2e/` 下建立项目根 `playwright.config.ts`
   - 浏览器：Chromium（headless，--font-render-hinting=none 确保字体清晰度回归）
   - 启动方式：与现有本地 dev 双进程并列（`scripts/start-local.sh` 起 trunk serve + cargo run，harness 复用 8080 / 8081）
   - WebSocket 捕获：`page.on('websocket')` 用于 ST-FE-S05-* presence / OT 验证

2. **56 用例映射**
   - ST-FE-S03-*（5）→ `auth.spec.ts`：`page.fill` 触发 login / register / refresh / logout / me
   - ST-FE-S04-*（6）→ `rooms.spec.ts`：房间列表 / 创建 / 邀请 / 加入 / 角色切换 / 离开
   - ST-FE-S05-*（6）→ `collab.spec.ts`：WS 握手 / presence / OT op 应用 / 冲突解决 / 断线重连 / ack
   - ST-FE-V2-*（4）→ `e2e-v2.spec.ts`：跨 S03~S05 全链路回归（auth → rooms → collab）
   - ST-FE-PROTO-*（8）→ `prototype-parity.spec.ts`：与 `core-01-editor-prototype.html` 像素相似度 ≥ 95%
   - ST-CR-01 / ST-MM-* / ST-PC-01 / ST-SP-01 / ST-UI-05 → 各自 spec

3. **OpenLogos reporter 集成**
   - `frontend-rs/tests/e2e/reporter/openlogos.ts` 把 playwright 结果转 `logos/resources/verify/test-results.jsonl` 格式
   - 每条 `status` 字段映射：`passed→pass`、`failed→fail`、`skipped→skip`、`timedout→fail`

4. **CI / 本地集成**
   - `scripts/run-verify-tests.sh` 阶段 2 新增 `e2e` 步骤
   - 阶段 3 不变（cargo test）

### 不在本变更范围

- Monaco wasm 完整挂载（implementation §2.3 标记 "可选升级"）
- `diagrams` 版本历史（V2 后端增量）
- `indices` 接收 frontend 写入（V2 后端增量）
- Kubernetes / 生产 TLS / Prometheus（V1 明确未实现）

## 风险

| 风险 | 缓解 |
|---|---|
| Playwright 引入新依赖（npm 包、Chromium 二进制） | 仅在 `frontend-rs/tests/e2e/` 下安装；不进主项目 `package.json` |
| Harness 启动耗时（~30s） | `openlogos verify` 增加 timeout；harness 默认不跑 S04/S05 视频会议模式 |
| 像素相似度对 HiDPI 屏敏感 | 强制 headless `--force-device-scale-factor=1` |
| 56 个用例一次性跑完失败率高 | 按 S03/S04/S05/V2/PROTO 分批（与 implementation A/B/C/D 对齐），逐批 PASS 后归档 |