# 变更提案：fix-add-frontend-stub-leftover

> module: core | created: 2026-06-11
> guard: logos/.openlogos-guard
> 前置: `fix-modal-overlay-blocking` 已归档（smoke 3/5，HP-02/03 揭示 2 个 add-frontend-completeness 范围 bug，详见 `logos/changes/archive/20260611-2126-fix-modal-overlay-blocking/` + `logos/spec/smoke-report.md`）

## 变更原因

`fix-modal-overlay-blocking` 提案闭环（modal 遮罩 / canvas testid / backend CORS 3 修复 + e2e 脚本修正），5 HP smoke 从 0/5 提升到 **3/5**（HP-01/04/05 PASS）。HP-02/03 失败根因经源码追踪后，**新发现 2 个 add-frontend-completeness 提案遗留的功能未接入 bug**，均不在 fix-modal-overlay-blocking 提案 scope 内（参见其 §"不在本提案范围"）：

**事实盘点**（`logos/spec/smoke-report.md` 5/5 现状，HP-02/03 失败错误日志）：

| # | 类别 | 描述 | 证据 |
|---|---|---|---|
| 1 | **save handler 空闭包 bug (Bug A)** | `editor_panels.rs:1115, 1125-1127, 1156, 1173` 4 处 `debouncer.schedule(move || {})` 是空闭包，1.1s 后**不发 PUT**；`on_save` 闭包内 `let _ = store.dirty.get();` 是唯一有 body 但 no-op 的 stub | HP-02 `expected >= 1 PUT, got 0` |
| 2 | **侧栏 selected 链路传错 (Bug B)** | `editor_panels.rs:715` LeftPanel 列表项 `on:click=move |_| { on_select(Some(testid_for_select.clone())); }` 把 `data-testid` 字符串（如 `"table-list-item-auto-12"`）传给 `selected_table_id`；RightPanel `selected_table` memo 用 `t.id == id` 比对 → 永远 None → 右栏 field 面板不显示、点中不高亮 | HP-03 `waiting for locator('[data-testid="btn-add-field"]')` 30s 超时 |

**根因**：
- Bug A: add-frontend-completeness B1~B5 在每个 handler 末尾**预留**了 `debouncer.schedule(...)` 调用，但闭包 body 写成了 `|| {}` 占位，没有真的接入 `DiagramClient::save` async 路径
- Bug B: `data-testid` 字符串与 `table.id` 命名空间混淆（testid 是 `table-list-item-{id}`，handler 不小心把整个 testid 当 id 传了），同时 `class:cdb-is-selected` 比较已用真 `table_id`，所以视觉与逻辑两症状同源

**附加影响**：Bug B 让 HP-03 无法验证 field add / change type / Share URL；Bug A 让 HP-02 无法验证 auto-save 持久化。这两个 bug 不在 fix-modal-overlay-blocking 范围（其 scope 明确"不在本提案范围"列了 4 个 stub 但**没列这 2 个**，属于"补提案"）。

## 变更类型

**代码级**（纯 bug fix，无功能新增 / 行为变更）：
- 4 处 `debouncer.schedule(move || {})` 替换为公共 `schedule_save()` helper（闭包内 `spawn_local` 调 `DiagramClient::save`）
- LeftPanel 列表项 `on:click` 传 `table.id` 替 `testid_for_select`（1 行 diff）
- `lib.rs` 解析 `window.location.pathname` 拿真 diagram_id，fallback `"default"`
- 抽 `is_table_selected()` 纯函数 + 3 UT + 强 e2e 断言

## 变更范围

- **影响的需求文档**：无（baseline 不变）
- **影响的功能规格**：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5.3.1 测试 ID 索引追加 UT-STUB-01/02 + ST-STUB-01
  - `prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`（如存在）— §LeftPanel 侧栏选中语义
  - `prd/2-product-design/1-feature-specs/core-08-data-sync.md`（如存在）— save 流程图
- **影响的业务场景**：S01（前端 UI 完整闭环）
- **影响的 API**：无（仅调用既有 `PUT /api/v1/diagrams/{id}`）
- **影响的 DB 表**：无
- **影响的编排测试**：无（仅 e2e）
- **影响的 e2e 测试**：`frontend-rs/scripts/e2e-smoke.mjs`（HP-02 强断言 PUT 数 + revision 推进；HP-03 加 `cdb-is-selected` class 数 + 右栏 h3 含表名）
- **影响的 smoke 报告**：重跑后会得到 `SMOKE_PASS` 而非 `SMOKE_FAIL`

## 部署影响

- **是否需要部署**：是
- **部署原因**：前端 save 链路未接通，无法持久化；侧栏选中链路 bug 让侧栏/右栏交互完全失效
- **影响环境**：本地 dev（沿用 fix-modal-overlay-blocking 的本地 dev + Playwright 方案）
- **是否涉及数据迁移**：否
- **是否需要回滚预案**：是（4 处 `debouncer.schedule` 改回空闭包 + 侧栏 `on:click` 改回 testid 即可纯 revert）
- **是否需要 smoke**：是（重跑 e2e-smoke.mjs 验证 5/5 PASS）

## 变更概述

本提案是 1 批次闭环：4 处 save handler 接 `spawn_local` + 公共 helper + 侧栏 1 行 diff + `is_table_selected` 纯函数 + `lib.rs` pathname 解析 + 3 UT + e2e 4 强断言 + 1 份测试用例规格 + 重跑 smoke。

| 批次 | 范围 | 闭环交付物 |
|---|---|---|
| **B1** | Bug A 修复（save handler ×4 + schedule_save helper + lib.rs pathname）+ Bug B 修复（LeftPanel 单行）+ 1 份 stub-leftover 测试用例规格 | Rust UT（is_table_selected + schedule_save 副作用契约）+ Playwright e2e 强断言 + OpenLogos reporter |

### 关键约束

1. `schedule_save` helper 抽公共（避免 4 处雷同），AppRoot 内私有 fn
2. `is_table_selected` 纯函数必须**禁止**接受 `data-testid` 形式（如 `table-list-item-xxx`），确保 Bug B 根因被函数契约阻断
3. e2e 强断言 4 项全加（HP-02 PUT 数 + revision；HP-03 selected class + 右栏 h3），保证 Bug A/B 永远被 smoke 捕获
4. `lib.rs` pathname 解析失败时 fallback `"default"`（与现状一致，最差 PUT 404 但不影响 UI shell）
5. 单批次必须 5/5 smoke PASS 才能 archive

### 风险与缓解

- **风险**：`schedule_save` 闭包跨越 `spawn_local` 边界捕获 `client` / `store` / `id` / `rev` / `snap`，需要 `Clone` 友好
  - **缓解**：`DiagramClient` 内部 `Rc`（`Clone` 友好）；`EditorStore` 内部全 `RwSignal`（`Clone` 友好）；`id: String` / `rev: i64` 是 `Copy` / `Clone`；`snap: Diagram` 已 derive `Clone`
- **风险**：`store.snapshot()` 调用需要在 wasm 端 run，与 `spawn_local` 在同一线程
  - **缓解**：`snapshot` 是同步纯函数（`editor_core.rs:150`），无问题
- **风险**：HP-02 读 `window.__cdb_revision` 需要在 `lib.rs` 暴露 store 到 window 做测试钩子
  - **缓解**：`#[cfg(debug_assertions)]` 限定，trunk release 构建自动剔除
- **风险**：e2e 4 强断言一旦加严，未来小修改可能让 smoke 不稳定
  - **缓解**：4 个断言都基于**已验证**的 store/UI 状态（PUT 数 = 实际请求数；revision 来自 `SaveResponse`；selected class 来自 leptos `class:`；h3 文本来自 `table_name`），无 flaky 风险

### 不在本提案范围

- `on_set_ref` 仍不接 save 路径（独立功能未实现，留给后续）
- `on_force_overwrite` / `on_reload` 仍 stub（409 conflict 不在主链路触发路径，留给后续）
- share URL 加载、undo/redo 实际 effect（同 fix-modal-overlay-blocking "不在本提案范围"）
- ConfigureCustomTypes session state（V1 spec §5.9 限制）
- `lib.rs` `window.location.pathname` 解析**仅取最后一段**（不处理 query string / hash）

## 5 大 Happy Path 现状

| HP | 修复前 | 修复后期望 |
|---|---|---|
| HP-01 Load blank editor | PASS（modal 修好） | 维持 PASS |
| HP-02 Auto-save + reload | FAIL（PUT=0，无 save 链路） | PASS（PUT≥1 + revision 推进 + reload 持久化） |
| HP-03 Field + share | FAIL（btn-add-field 30s 超时） | PASS（selected class=1 + 右栏 h3 含表名 + field add/type/share URL） |
| HP-04 SQL import parse | PASS（modal 修好） | 维持 PASS |
| HP-05 Keyboard shortcuts | PASS（modal + testid 修好） | 维持 PASS |

## 与 fix-modal-overlay-blocking 提案的关系

- 本提案**不阻塞** fix 提案 archive（C1 路径：fix 已先 archive，本提案后 archive）
- 本提案目标是 5/5 smoke PASS，让 fix-modal-overlay-blocking 满足 §"关键约束 #4"
- 计划文件：`/root/.claude/plans/rustling-fluttering-yeti.md`
