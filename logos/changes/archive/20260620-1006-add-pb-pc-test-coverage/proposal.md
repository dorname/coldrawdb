# 变更提案：add-pb-pc-test-coverage

> module: core | created: 2026-06-20

## 变更原因

2026-06-20 运行 `openlogos verify` 时观察到 Gate 3.5 失败：覆盖度 82%（65/79），**14 个 pre-existing 用例未覆盖**：

| 用例 ID | 涉及模块 | 数量 |
|---|---|---|
| UT-PB-01 ~ UT-PB-05 | Phase B 关系工具 | 5 |
| ST-PB-01 | Phase B 关系工具（E2E） | 1 |
| UT-PC-01 ~ UT-PC-06 | Phase C 导入/导出 | 6 |
| ST-PC-01 | Phase C 导入/导出（E2E） | 1 |
| UT-AB-04 | AppBar 回归（PC 引入 enabled 断言） | 1 |
| **合计** | | **14** |

**根因诊断**（grep 已验证）：
- 所有 PB / PC 相关函数实现完整：`build_reference` / `flip_reference_endpoints` / `toggle_field_primary` / `parse_sql_statements` / `export_diagram_sql` / `export_diagram_dbml` / `count_dbml_tables` / `open_import_drawer` 均位于 `frontend-rs/src/editor_panels.rs`；`hit_test_field` 位于 `frontend-rs/src/editor_render.rs`
- UI 字符串 `guide-import-sql` / `btn-import` / `io-drawer` / `inspector_open` 均存在
- 但**对应的测试代码文件从未创建**（`frontend-rs/tests/` 下只有 6 份 Rust 测试，均与 PB/PC 无关；`frontend-rs/tests/e2e/` 下 15 份 Playwright spec 也未覆盖 PB/PC）

**用例定义完整**：3 份测试用例文档（`core-PB-relationship-test-cases.md` / `core-PC-import-export-test-cases.md` / `core-UI-modals-2-test-cases.md`）均已落地，每个用例含 Given/When/Then 与对齐实现路径，无需新增 spec delta。

**本次变更旨在补齐缺失的测试代码**，让 Gate 3.5 通过 100% 覆盖度，恢复 verify 绿灯。

## 变更类型

**代码级修复（新增测试代码）**

- 不涉及业务代码（`src/`、`backend/`、`frontend-rs/src/` 实现不动）
- 不涉及 API 契约变更
- 不涉及 DB Schema 变更
- 不涉及部署变更
- 不涉及 spec 文档变更（用例定义已完整）

## 变更范围

### 影响的需求文档：**无**
### 影响的功能规格：**无**
### 影响的业务场景：**无场景 ID 变更**
### 影响的部署方案：**无**
### 影响的 API：**无**
### 影响的 DB 表：**无**
### 影响的编排测试：**无**
### 影响的 smoke 测试：**无**

### 唯一变更：新增 4 份测试文件

| # | 文件路径 | 覆盖用例 | 类型 |
|---|---|---|---|
| 1 | `frontend-rs/tests/phase_b_relationship.rs` | UT-PB-01 ~ UT-PB-05（5 个） | Rust 单元测试 |
| 2 | `frontend-rs/tests/phase_c_import_export.rs` | UT-PC-01 ~ UT-PC-06 + UT-AB-04（7 个） | Rust 单元测试 |
| 3 | `frontend-rs/tests/e2e/16_relationship_tool.spec.ts` | ST-PB-01（1 个） | Playwright E2E |
| 4 | `frontend-rs/tests/e2e/17_import_drawer.spec.ts` | ST-PC-01（1 个） | Playwright E2E |

## 部署影响

- 是否需要部署：**否**
- 部署原因：本次变更仅在 `frontend-rs/tests/` 下新增测试代码文件，不影响任何运行时行为
- 影响环境：**无**
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**（测试代码可通过 git revert 回滚）
- 是否需要 smoke：**否**

## 变更概述

### 工作量估算

| 用例 | 实现位置（已 grep 验证） | 测试类型 | 估算 LOC |
|---|---|---|---|
| UT-PB-01 `hit_test_field` | `editor_render.rs::hit_test_field` | UT（纯函数） | ~30 |
| UT-PB-02 `build_reference` | `editor_panels.rs::build_reference` | UT（纯函数） | ~25 |
| UT-PB-03 `flip_reference_endpoints` | `editor_panels.rs::flip_reference_endpoints` | UT（纯函数） | ~20 |
| UT-PB-04 `toggle_field_primary` | `editor_panels.rs::toggle_field_primary` | UT（纯函数） | ~25 |
| UT-PB-05 UI 确认条 | `editor_panels.rs` 组件状态 | UT（信号 + 组件） | ~40 |
| ST-PB-01 关系工具双点 | `editor_panels.rs` + Playwright | E2E | ~60 |
| UT-PC-01 `parse_sql_statements` | `editor_panels.rs::parse_sql_statements` | UT（纯函数） | ~25 |
| UT-PC-02 `export_diagram_sql` | `editor_panels.rs::export_diagram_sql` | UT（纯函数） | ~30 |
| UT-PC-03 `export_diagram_dbml` | `editor_panels.rs::export_diagram_dbml` | UT（纯函数） | ~30 |
| UT-PC-04 `open_import_drawer()` | `editor_panels.rs` 组件状态 | UT（信号 + 组件） | ~35 |
| UT-PC-05 `count_dbml_tables` | `editor_panels.rs::count_dbml_tables` | UT（纯函数） | ~15 |
| UT-PC-06 `guide-import-sql` | `editor_panels.rs` 组件 | UT（信号 + 组件） | ~35 |
| UT-AB-04 `btn-import` enabled | `editor_panels.rs` AppBar 组件 | UT（信号 + 组件） | ~30 |
| ST-PC-01 完整 E2E | `editor_panels.rs` + bridge | E2E | ~80 |

**总计**：约 480 行新增测试代码 + 用例映射注释。

### 实施顺序

按依赖关系由内到外：
1. **纯函数 UT**（PB-01~04，PC-01~03/05）：不依赖组件状态，单独可测
2. **组件 UT**（PB-05，PC-04/06，AB-04）：依赖 Leptos signal / 组件，需引用 `editor_panels` 模块
3. **E2E**（ST-PB-01，ST-PC-01）：依赖前后端全链路 + Playwright 配置

### 设计哲学

- **不重写业务代码**：仅新增测试文件，不动 `editor_panels.rs` / `editor_render.rs` 任何行
- **不新增 fixture**：复用现有测试 setup（如 `verify_bootstrap.rs` 风格），不引入新测试框架
- **遵循 cargo test 约定**：测试代码放 `frontend-rs/tests/`（顶层集成测试）或模块内 `#[cfg(test)] mod tests`
- **Playwright 一致性**：E2E 文件命名遵循现有 `01_xxx ~ 15_xxx.spec.ts` 序号，命名 `16_relationship_tool.spec.ts` / `17_import_drawer.spec.ts`
- **明确回归断言**：UT-AB-04 在 PC 引入的回归断言中明确（替换 Phase A disabled → Phase C enabled），测试需双断言

## 关键风险与缓解

| 风险 | 缓解 |
|---|---|
| `editor_panels.rs` 是 Leptos 组件文件，函数可见性受限 | 若关键函数为 `pub(super)` 或私有，测试代码需通过 `pub(crate)` 暴露或使用 `#[cfg(test)] pub fn` 内联测试 |
| `parse_sql_statements` 等 SQL 解析函数依赖 `sqlparser` crate 行为 | UT 用最小化 SQL fixture（与 `phase3_bridge.rs` 测试一致） |
| Playwright E2E 依赖完整前后端启动 | 复用现有 `_setup.sh` / `_teardown.sh`，与 `04_change_type.spec.ts` 模式对齐 |
| UT-AB-04 与 PC-04 共享 `btn-import` 触发逻辑 | 测试代码解耦（AB-04 断言 `enabled` 状态，PC-04 断言 `open_import_drawer` 副作用），不互相依赖 |