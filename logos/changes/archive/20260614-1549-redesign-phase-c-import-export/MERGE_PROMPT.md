# 合并指令

## 变更提案
- 提案名称：redesign-phase-c-import-export
- 提案目录：logos/changes/redesign-phase-c-import-export/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/redesign-phase-c-import-export/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
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
   git add -A && git commit -m "docs(redesign-phase-c-import-export): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive redesign-phase-c-import-export`。
