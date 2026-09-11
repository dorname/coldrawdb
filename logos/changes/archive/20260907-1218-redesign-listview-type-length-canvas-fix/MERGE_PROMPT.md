# 合并指令

## 变更提案
- 提案名称：redesign-listview-type-length-canvas-fix
- 提案目录：logos/changes/redesign-listview-type-length-canvas-fix/

## 提案内容

# 变更提案：redesign-listview-type-length-canvas-fix

> module: core | created: 2026-09-07

## 变更原因

用户真机反馈 + PDManer 对标诉求，三项工作一条变更线：

1. **列表视图重设计（对标 PDManer）**：现 ListView 是只读分组清单（过滤/排序/双击跳画布），不具备字段编辑能力。目标对齐 PDManer 字段明细表格：行内编辑字段名/类型/长度/小数位/约束（PK/不为空/唯一/自增）/说明，增删字段、上下移动，按表切换。
2. **PostgreSQL 类型对齐 + VARCHAR 长度配置**：类型下拉当前是整串枚举（`VARCHAR(255)`/`DECIMAL(10,2)` 写死），长度不可配。PDManer 口径：数据类型（基类型）+ 长度列 + 小数位数列分离配置。Field.type_ 已是自由字符串（无 size/precision 字段，`editor_core.rs:64-80`），最小改法 = type_ 存参数化整串（如 `VARCHAR(32)`），UI 层解析/合成。**执行顺序：先调原型后调代码。**
3. **bug 修复**（根因已勘察确认）：
   - **tag 输入丢字**：Inspector 单体动态块（`editor_panels.rs:4369/4411`）订阅 `store.tables`，blur 写回（`:4469`）无条件 notify 导致表单整体重建；`prop:value=field.tag.clone()`（`:4462`）是静态快照，未提交文本随重建丢失。
   - **note/area 无法拖动**：拖拽状态机无 note/area 分支——命中后只选中（`editor_render.rs:760-779`），`DragState`（`:280-318`）无对应变体，pointermove/up 无处理；且 `on_update_area`（`editor_panels.rs:8506`）无 x/y 写回通道。
   - **卡死**：非死循环，是高频 signal set × 同步全量重绘叠加——endpoint 拖动每 mousemove `references.set`、便签/区域每击键 `notes/areas.set`，均触发绕过 rAF 去重的全量重绘 + 表单重建 + 每击键全量 snapshot。

## 变更类型

设计级变更（功能规格 + 原型 + 测试用例 + 代码）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `core-04-side-panel-tabs.md`（ListView 重设计：字段明细行内编辑网格）
  - `core-01a-table-and-field.md`（§2.2 增补参数化类型口径：基类型 + 长度/小数位；PG 对齐）
  - `core-S04-room-lifecycle-design.md`（如涉及引擎清单口径引用，仅指针性更新）
- 影响的原型：`core-01-editor-prototype.html`（**先行**：renderListView 重设计为字段明细编辑表格 + 类型/长度/小数位分离配置）
- 影响的业务场景：SP（侧栏/列表视图）
- 影响的部署方案：无
- 影响的 API：无（type_ 整串落账，后端无感）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 ST 用例：
  - `core-SP-side-panel-test-cases.md`：ST-SP-LIST-01（MODIFIED：列表视图从只读清单升级为字段明细编辑网格）、ST-LV-01（MODIFIED：分组/编辑断言）
  - `core-UI-modals-2-test-cases.md`：UT-MM-36（MODIFIED：参数化类型解析/合成纯函数）、ST-DB-02（回归校验 PG 清单）
  - 新增：ST-SP-LIST-02（行内编辑落账）、tag 输入/note/area 拖动回归用例
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端变更（ListView 重设计 + 类型参数化 UI + canvas 交互修复），无后端/部署方案变更
- 影响环境：无
- 是否涉及数据迁移：否（type_ 整串参数化向后兼容：老数据 `VARCHAR(255)` 等可正常解析）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **ListView PDManer 化（先原型后代码）**：列表视图从只读分组清单升级为字段明细编辑网格——列：字段代码/显示名称（comment）/主键/不为空/唯一/自增/数据类型/长度/小数位/说明（tag）；行内编辑直接落账；工具条：增字段、删字段、上移/下移；按表分组切换或表选择器。画布/Inspector 保持既有能力不变。
2. **类型参数化**：类型下拉改为基类型（VARCHAR/INT/DECIMAL…按引擎过滤），长度列（VARCHAR 系可编辑，默认 255）、小数位列（DECIMAL/NUMERIC 系可编辑，默认 10,2）；type_ 落账为合成整串（`VARCHAR(32)`、`NUMERIC(10,2)`）。纯函数 `parse_field_type` / `compose_field_type` 承担解析/合成，UT 覆盖。PG 类型清单保持 core-01a §2.2 权威表口径。
3. **bug 修复**：tag 输入改受控输入模式（本地 draft signal + 响应式 `prop:value`，同文件 `:6002/:2874` 已有正确范式），消除输入丢字；仿表拖动三段式补 note/area 的 DragState 变体与 pointermove/up 链路，补 `on_update_area` x/y 写回通道；高频写回治理——输入防抖写回、拖拽期只写视觉层（mouseup 才落账）、paint 走 rAF 合并、snapshot 移入 debounce 回调，消除卡死。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/test/core-SP-side-panel-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/test/core-UI-modals-2-test-cases.md

- Delta 文件：`logos/changes/redesign-listview-type-length-canvas-fix/deltas/test/core-UI-modals-2-test-cases.md`
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
   git add -A && git commit -m "docs(redesign-listview-type-length-canvas-fix): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive redesign-listview-type-length-canvas-fix`。
