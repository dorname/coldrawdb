# 合并指令

## 变更提案
- 提案名称：fix-overflow-menu-room-delete-listview-io
- 提案目录：logos/changes/fix-overflow-menu-room-delete-listview-io/

## 提案内容

# 变更提案：更多菜单边界保护 + 房间列表删除入口 + ListView 导入导出

> module: core | created: 2026-09-07

## 变更原因

用户实测反馈（2026-09-07，附截图）：

1. AppBar「更多菜单」下拉（导入/导出/分享设置/设置/删除图表/浅色模式/命令面板 共 7 项）超出显示边界——窗口高度不足时底部菜单项被裁切（截图中「命令面板」已被切半），右缘也可能贴死视口边缘。根因：`.cdb-app-bar__overflow-menu` 只做 `right: 0` 锚定，无 `max-height` / 视口边界保护。
2. 协作空间「无法删除」——删除能力其实已存在（后端 `DELETE /rooms/{id}` 软删除 + 编辑器内 RoomPanel「删除房间」），但**房间列表页卡片上没有任何删除入口**，用户在列表页无从删起。
3. 列表视图（ListView）工具栏只有字段操作与「返回画布」，无导入/导出入口；用户需要在列表视图同样可用 IO 能力。

「连接数据库导入」（DB introspection 导入）与「DDL 真实数据库回读验证」拆分为后续独立提案 B，不在本提案范围。

## 变更类型

设计级变更（新增交互入口 + 既有样式缺陷修复；无 API/DB 结构变更——房间删除复用既有 `DELETE /rooms/{id}`）

## 变更范围

- 影响的需求文档：无（不改变场景定义）
- 影响的功能规格：
  - `core-05-top-menu-modals.md` — 更多菜单视口边界保护条款
  - `core-S04-room-lifecycle-design.md` — 房间列表页卡片删除入口（owner 可见）
  - `core-01d-import-export.md` — IO 抽屉入口扩展至列表视图
  - `core-04-side-panel-tabs.md` — ListView 工具栏加导入/导出入口
  - `core-00-information-architecture.md` — IA 同步（如入口位置有变）
- 影响的业务场景：S04（房间生命周期）、Phase C（导入导出）
- 影响的部署方案：无
- 影响的 API：无新增；复用 `DELETE /api/v1/rooms/{id}`
- 影响的 DB 表：无
- 影响的编排测试：无（非 API 编排变更）
- 影响的 smoke 测试：无
- 影响的测试用例文档：
  - `core-S04-test-cases.md` — 新增房间列表页删除 e2e 用例
  - `core-PC-import-export-test-cases.md` — 新增 ListView IO 入口用例
  - 菜单边界保护新增 e2e 用例（小视口菜单不裁切）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 UI/交互变更 + 复用既有后端删除端点，无部署物变化
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **更多菜单边界保护**：`.cdb-app-bar__overflow-menu` 增加 `max-height: calc(100vh - 锚定偏移)` + `overflow-y: auto`，右缘夹取保证不超出视口；原型 HTML 同步。
2. **房间列表删除入口**：rooms-list-page 房间卡片增加 owner 可见的删除按钮，点击弹确认模态（复用 DeleteRoom 语义与 `can_delete_room` 权限口径），确认后调 `DELETE /rooms/{id}` 并从列表移除；非 owner 不显示。
3. **ListView 导入导出入口**：ListView 工具栏「返回画布」旁新增「导入」「导出」按钮，打开与画布态完全相同的 IoDrawer（Import/Export），导入走上一变更已落地的本地合并管道（`merge_import_into_store`），导入成功 Toast 提示并落账；只读（Viewer）下禁用导入。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/test/core-PV-verify-pre-run-test-cases.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/test/core-PV-verify-pre-run-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/fix-overflow-menu-room-delete-listview-io/deltas/test/core-S04-test-cases.md`
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
   git add -A && git commit -m "docs(fix-overflow-menu-room-delete-listview-io): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-overflow-menu-room-delete-listview-io`。
