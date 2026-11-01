# 变更提案：fix-e2e-harness-stability

> module: core | created: 2026-11-01
> 关联 issue：#67（ST-PU-27 验收负载时序抖动）、#68（G 批 goto 字体挂起假失败）

## 变更原因

正式验收（`scripts/run-verify-tests.sh` → Gate 3.6）近两轮暴露两个 e2e 测试基建稳定性缺陷（均为假失败，产品代码无回归）：

1. **#68**：G 批 Playwright TS specs（`frontend-rs/tests/e2e/`）的 `page.goto("/", { waitUntil: "load" })` 无外部字体 abort 兜底，离线/受限网络下 `load` 事件被 fonts.googleapis.com 挂起 → 30s 超时假失败（本轮 ST-PB-05 即此因）。parity 系列手写脚本均有 abort，唯独 TS specs 基建未对齐。
2. **#67**：E 批 ST-PU-27 是全仓唯一「进房后立即拖拽」用例，验收负载下交互落进页面初始化窗口 → 拖拽处理器未生效，宽度恒 330 假失败。真实栈仪器化诊断证明功能正确；隔离重跑三连 PASS。

来源：`feat-room-diagram-picker` 验收轮的根因排查（见 `.github/issue-drafts/` 两份草稿，已提交为 #67/#68）。

## 变更类型

代码级（仅 e2e 测试基建，不改产品代码、不改用例断言语义）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无（ST-PB-05 / ST-PU-27 用例定义与断言语义不变）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试基建：
  - `frontend-rs/tests/e2e/playwright.config.ts`（G 批：全局字体 abort）
  - `frontend-rs/scripts/test-spec-parity-e.mjs`（#67：ST-PU-27 拖拽前稳态等待）

## 部署影响

- 是否需要部署：否
- 部署原因：纯测试基建修改，不影响任何运行时产物（backend / frontend / MCP）
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **#68**：在 `frontend-rs/tests/e2e/playwright.config.ts` 增加全局 route abort（fonts.googleapis.com / fonts.gstatic.com），一处修复覆盖全部 TS specs；与 parity 系列脚本的外部资源隔离口径对齐。不改变任何用例的等待与断言语义。
2. **#67**：ST-PU-27 在 `createRoomAndEnter` 与首次 `dragSplitter` 之间增加稳态等待（轮询确认无 in-flight 请求 + 首帧渲染稳定），消除「进房后立即交互」与初始化窗口的竞态；两条钳制断言保持不变。其余 E 批用例不受影响（仅动 ST-PU-27 自身的准备段）。

验收口径：`run-verify-tests.sh` 全量重跑 Gate 3.6 PASS，且 G 批/E 批无导航或拖拽类假失败；产品代码零改动（`git diff` 仅触及上述两个测试基建文件）。
