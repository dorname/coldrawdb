# 变更提案：add-baseline-docs

> module: core | created: 2026-06-12

## 变更原因

项目已完成 Phase 4（React 前端全面替换为 Rust Web/WASM），近期又相继归档 `add-frontend-completeness`、`fix-modal-overlay-blocking`、`fix-add-frontend-stub-leftover` 三个变更。但基线文档存在以下滞后：

1. `README.md` 中后端端口仍写 `127.0.0.1:6666`，而当前 dev 环境已统一为 `:3000`；同时仍保留大量 React 下线说明，冲淡核心信息。
2. `AGENTS.md` 缺少当前模块状态、最近归档变更索引，以及 OpenLogos 快速链接，新成员难以快速判断项目所处阶段。
3. `logos/resources/reference/` 目录仅有一个 `.gitkeep`，缺少术语表、模块清单、开发环境速查等基线参考。

为避免后续迭代因文档不一致产生理解偏差，需统一刷新上述基线文档。

## 变更类型

设计级

## 变更范围

- 影响的需求文档：无（本次不改动业务需求）
- 影响的功能规格：无（本次不改动产品功能规格）
- 影响的业务场景：无
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的基线文档：
  - `README.md`
  - `AGENTS.md`
  - `logos/resources/reference/core-baseline-reference.md`（新增）

## 部署影响

- 是否需要部署：否
- 部署原因：本次仅更新项目基线文档，不修改任何运行时代码或配置
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 更新 `README.md`：将后端端口修正为 `:3000`，精简 React 下线冗余描述，补充当前技术栈摘要、本地启动命令、常用验证接口。
2. 更新 `AGENTS.md`：在现有 OpenLogos 方法论说明基础上，增加 `core` 模块状态（`lifecycle: launched`）、最近归档变更索引、以及 `openlogos next` / `openlogos status` 快速链接。
3. 新增 `logos/resources/reference/core-baseline-reference.md`：包含术语表、前端/后端模块清单、开发环境速查表、常用 CLI 命令、重要文件索引。
