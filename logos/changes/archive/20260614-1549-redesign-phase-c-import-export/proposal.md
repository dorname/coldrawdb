# 变更提案：redesign-phase-c-import-export

> module: core | created: 2026-06-14
> 前置：`redesign-phase-b-relationship` 已归档（Phase B 关系工具 + Inspector 增强已落地）

## 变更原因

Phase A 将 AppBar 的 `btn-import` / `btn-export` 提升为常驻按钮，但当前实现仍为占位：

1. **`btn-import` disabled**，空白引导「导入 SQL」不可用，用户无法从主路径导入 schema。
2. **`btn-export` 点击仅 toast 错误**，无 SQL/DBML/JSON 预览与复制/下载。
3. **File 菜单「导入」仍打开居中模态**（`ImportModal`），与 V2「非模态侧边抽屉」信息架构不一致。
4. 已有 `parse_sql_statements`、`ImportModal` UI shell、`POST /api/v1/bridge/import/local` 能力未贯通到可交付流程。

Phase C 将导入/导出收敛为**右侧 IO 抽屉**（非模态），与 Inspector 互斥展开，复用 bridge 导入 API，导出在客户端生成预览。

## 变更类型

设计级 + 代码实现（IO 抽屉 UI + 导入提交 + 导出预览）

## 变更范围

- **新增** `core-01d-import-export.md` — 导入/导出抽屉完整规格
- `core-05-top-menu-modals.md` — AppBar 按钮与 File 菜单接线；Import 模态降级
- `core-01-editor-canvas.md` — EmptyGuide「导入 SQL」启用
- `core-00-information-architecture.md` — IO 抽屉布局与 z-index
- `core-03-bridge-io.md` — 客户端导出预览与 bridge 导入对接说明
- **新增** `core-01-editor-prototype.html` delta — 抽屉视觉原型
- **新增** `core-PC-import-export-test-cases.md`
- 代码：`editor_panels.rs`、`editor_data_access.rs`（import 提交）、`styles.css`

## 影响的业务场景

- 画布空白引导 → 导入 SQL（EmptyGuide）
- AppBar 项目 IO → 导入/导出抽屉
- File 菜单 → 导入（改开抽屉，保留 New/Open 模态）

## 影响的 API

- `POST /api/v1/bridge/import/local` — 导入抽屉提交（已有，接线）
- `GET /api/v1/bridge/import/local/logs` — 可选轮询任务状态（V1 同步成功路径优先）

## 影响的 DB 表

- 无 schema 变更（复用 `task` 表记录导入任务）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 WASM 交互增强 + 已有 bridge API 接线；无后端/API/DB 变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否（本地 UT + e2e 增补 ST-PC-01）

## 变更概述

### Phase C 交付物

1. **ImportDrawer**（`data-testid="import-drawer"`）：格式 Tab（SQL/DBML/JSON）、引擎选择、粘贴区/文件拖放、实时解析摘要、提交导入。
2. **ExportDrawer**（`data-testid="export-drawer"`）：格式 Tab、引擎选择（SQL）、只读预览、复制/下载。
3. **AppBar 接线**：`btn-import` 启用；`btn-export` 打开导出抽屉；File→导入 同步开抽屉。
4. **EmptyGuide**：「↑ 导入 SQL」启用，打开 ImportDrawer。
5. **互斥规则**：IO 抽屉打开时自动折叠 Inspector；关闭抽屉恢复先前 Inspector 状态。
6. **V1 边界**：SQL/DBML 全屏代码视图（`btn-code-view`）仍 Phase D；Mermaid/PNG 导出不纳入本阶段。
