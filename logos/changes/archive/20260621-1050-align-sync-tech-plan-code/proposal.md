# 变更提案：align-sync-tech-plan-code

> module: core | created: 2026-06-21

## 变更原因

`sync-tech-plan-with-product-design` 规格合并后，代码仍与 Phase 3 时序图存在三处缺口：`?share=` URL 解析、E4 Command Palette / Code View 交互、保存失败指数退避（3s/6s/12s）。

## 变更类型

代码级变更（前端 `frontend-rs`）

## 变更范围

- 场景：S01（保存重试 + E4）、S02（`?share=` 解析）
- 文件：`lib.rs`、`editor_data_access.rs`、`command_palette.rs`、`code_view.rs`、`editor_panels.rs`
- 测试：新增 UT + OpenLogos reporter 条目

## 部署影响

- 是否需要部署：否（本地 dev / staging 常规发布即可）
- 是否需要 smoke：否

## 变更概述

1. `lib.rs`：`?share=` 优先于 pathname 解析 diagram id
2. `editor_data_access.rs`：`save_with_retry`（3s/6s/12s，封顶 30s；409 不重试）
3. E4：`CommandPalette`（Ctrl+K + 搜索 + Enter 选中）与 `CodeView`（SQL/DBML/JSON + 复制）
4. `AppRoot` / `AppBar` 接线 + 离线保存状态 UI
