# 变更提案：fix-modal-overlay-blocking

> module: core | created: 2026-06-10
> guard: logos/.openlogos-guard
> 前置: `add-frontend-completeness` 已归档（smoke 0/5 FAIL，详见 `logos/changes/archive/20260610-2122-add-frontend-completeness/` + `logos/spec/smoke-report.md`）

## 变更原因

`add-frontend-completeness` 提案 B1~B5 实施完毕并 verify PASS，但本地 dev 部署 + Playwright 5 HP smoke 跑出 **0/5 FAIL**（2026-06-10）。

**事实盘点**（`logos/spec/smoke-report.md` 5/5 失败错误日志）：

| # | 类别 | 描述 | 证据 |
|---|---|---|---|
| 1 | **真实 UI bug** | `ModalRoot`（`editor_panels.rs:1456`）遮罩 `<div class="cdb-modal-overlay">` 无条件渲染，未受 `kind.get().is_some()` 控制，**永远挡在所有按钮之上** | 5/5 HP 全部 `intercepts pointer events` 错误 |
| 2 | **Smoke 脚本误用 testid** | `e2e-smoke.mjs` 用 `[data-testid="editor-canvas"]`，但代码里画布是 `cdb-canvas-container` / `cdb-canvas-empty`，**没有 testid** | HP-01 / HP-05 找不到 `editor-canvas` |
| 3 | **预存在 CORS 缺失** | backend `:6666` 未配 `actix-cors` middleware，frontend `:8080` 跨源 PUT 触发 CORS preflight 失败，HP-02 即便能点击也无法真正保存 | HP-02 PUT 被拦截（不在本提案 scope，但 smoke 需要它通过） |

**根因**：`ModalRoot` 内部 `{move || match kind.get() { ... }}` 只条件渲染了**模态体**，但**遮罩 div 自身是无条件**的，leptos view 树永远把它放进 DOM。后果：模态关闭后遮罩仍占满 viewport，pointer events 全部吞掉。

**附加影响**：这一 bug 同时让 `frontend-rs/tests/e2e/` 下已存在的 15 个 spec 也跑不通（`01_create_table` 起就点不到任何东西）。所以**新增 1 个 testid 补在画布上是相辅相成的**，让 e2e 能定位画布区域。

## 变更类型

**代码级**（纯 bug fix，无功能新增 / 行为变更）：
- ModalRoot 遮罩 div 改成受 `kind.get().is_some()` 控制
- 画布容器补 `data-testid="editor-canvas"`
- backend main.rs 加 CORS middleware（actix-cors crate + 配置 origin）

## 变更范围

- **影响的需求文档**：无（baseline 不变）
- **影响的功能规格**：
  - `prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 修一处描述：「模态打开时显示遮罩」需明确为「仅模态打开时显示遮罩」
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5 渲染策略加一条「画布 testid = editor-canvas」
- **影响的业务场景**：S01 / S02（前端 UI 可用性恢复）
- **影响的 API**：无（仅 CORS middleware 是透传）
- **影响的 DB 表**：无
- **影响的编排测试**：无（仅 e2e）
- **影响的 e2e 测试**：`frontend-rs/scripts/e2e-smoke.mjs`（更新 testid 引用）
- **影响的 smoke 报告**：重跑后会得到 `SMOKE_PASS` 而非 `SMOKE_FAIL`

## 部署影响

- **是否需要部署**：是
- **部署原因**：前端 modal 不可用，UI 实际处于 broken 状态；backend 需要重启加载 CORS middleware
- **影响环境**：本地 dev（沿用 add-frontend-completeness 的本地 dev + Playwright 方案）
- **是否涉及数据迁移**：否
- **是否需要回滚预案**：是（modal 改回去 / 删 CORS middleware / 删 testid 即可纯 revert）
- **是否需要 smoke**：是（重跑 e2e-smoke.mjs 验证 5/5 PASS）

## 变更概述

本提案是 1 批次闭环：修 ModalRoot 条件渲染 + 补 canvas testid + 加 backend CORS，配套 1 份测试用例规格 + smoke 脚本 testid 修正 + 重跑 smoke。

| 批次 | 范围 | 闭环交付物 |
|---|---|---|
| **B1** | ModalRoot 条件渲染（frontend）+ editor-canvas testid（frontend）+ actix-cors middleware（backend）+ 1 份 fix 测试用例规格 | Rust UT + Playwright e2e 重跑 + OpenLogos reporter |

### 关键约束

1. ModalRoot 修改必须**最小化**：只动遮罩 div 的条件渲染，不动内部模态体
2. testid 加在 `cdb-canvas-container`（已有的外层 div），不引入新 wrapper
3. CORS middleware 配成允许 `http://localhost:8080`（dev 模式 origin），生产用 env 变量可覆盖
4. 单批次必须 5/5 smoke PASS 才能 archive

### 风险与缓解

- **风险**：ModalRoot 改成条件渲染后，B4 已写的背景点击关闭 / ESC 关闭逻辑失效
  - **缓解**：保留原有 on:click handler，只把 div 包一层 `if kind.get().is_some()`；行为等价
- **风险**：actix-cors crate 没在 Cargo.toml
  - **缓解**：dev dep 1 行新增，feature flag 默认开
- **风险**：CORS 配成 * 太宽
  - **缓解**：明确只允许 `http://localhost:8080`（dev mode origin），后续 production 由 env 变量覆盖

### 不在本提案范围

- 修复 add-frontend-completeness 留下的其他 stub：
  - share URL 加载（前端不读 share query param）
  - import 模态 submit handler 无 on:click
  - undo/redo 实际 effect（handler 是 `|| {}`）
  - set_ref 实际 effect（handler是 error toast）
- 这些 stub 在 smoke 里通过只测 UI shell 的方式绕开；如果产品要真正用，需要后续专门提案

## 5 大 Happy Path 现状

| HP | 修复前 | 修复后期望 |
|---|---|---|
| HP-01 Load blank editor | FAIL（editor-canvas testid 缺失） | PASS（testid 补上） |
| HP-02 Auto-save + reload | FAIL（modal 遮罩 + CORS preflight） | PASS（modal 修好 + CORS 配好） |
| HP-03 Field + share | FAIL（modal 遮罩） | PASS（modal 修好；share URL 仍只校验格式，load 由后续提案） |
| HP-04 SQL import parse | FAIL（modal 遮罩） | PASS（modal 修好；submit handler 仍 stub） |
| HP-05 Keyboard shortcuts | FAIL（modal 遮罩 + editor-canvas testid） | PASS（modal 修好 + testid 补上） |
