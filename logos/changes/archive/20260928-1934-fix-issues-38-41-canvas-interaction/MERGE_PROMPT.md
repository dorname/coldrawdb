# 合并指令

## 变更提案
- 提案名称：fix-issues-38-41-canvas-interaction
- 提案目录：logos/changes/fix-issues-38-41-canvas-interaction/

## 提案内容

# 变更提案：修复 #38-#41 画布交互四件套（撤销失效 / 选中保持平移 / 标签字号 / 区域锁定）

> module: core | created: 2026-09-28

## 变更原因

GitHub 四个 open issue 均属画布交互域：
- **#38 [BUG]** 拖动画布（pan）后 Ctrl+Z / Ctrl+Y 失效——撤销链路被 pan 交互破坏
- **#39 [Enhancement]** 选中表后空白拖动应平移画布且**保留选中**；当前 pointerdown 空白即清选（editor_render.rs 空白分支无条件 `selected_*.set(None)`）
- **#40 [Enhancement]** 远缩放下表名/字段名/注释不可读；需标签字号可调 + 屏幕空间最小字号钳制
- **#41 [Enhancement]** 区域（Area）支持锁定：锁定后不可拖动/改尺寸，状态随图表持久化

## 变更类型

设计级变更（#39/#40/#41 涉及交互合同与原型口径；#38 为代码级修复但同域一并处理；#41 涉及 DB migration → 含部署影响）

## 变更范围

- 影响的需求文档：无（issue 即需求来源，验收条件进功能规格）
- 影响的功能规格：`core-01-editor-canvas.md`（新增 §5.14 pan-undo 合同 / §5.15 选中保持平移 / §5.16 标签字号 / §5.17 区域锁定）
- 影响的业务场景：S01（持久化快照含 area.locked）；时序图不变（仅快照字段增量）
- 影响的部署方案：`core-01-deployment-plan.md`（migration 启动自动执行先例同 0007；smoke 增补区域锁定持久化断言）
- 影响的 API：无新端点；`AreaDto` 增 `locked` 字段（serde default false，旧图兼容）
- 影响的 DB 表：`area` 表新增 `locked` 列（migration 0012_area_lock，启动自动执行）
- 影响的编排测试：无（无新 HTTP 端点语义）
- 影响的 smoke 测试：`logos/resources/test/smoke/`（增补区域锁定持久化冒烟）
- 影响的测试文档：`core-KB-shortcut-test-cases.md`（#38 UT/ST）、`core-CR-canvas-test-cases.md`（#39/#40/#41 UT/ST）

## 部署影响

- 是否需要部署：是
- 部署原因：#41 后端 migration `0012_area_lock`（area 表加 locked 列）+ AreaDto 变更，需部署后端生效
- 影响环境：本地 / 生产（compose）
- 是否涉及数据迁移：是（启动自动 migration，既有 area 行 locked 默认 0=未锁定，幂等）
- 是否需要回滚预案：否（加列默认值，后向兼容）
- 是否需要 smoke：是（区域锁定持久化：锁定 → 保存 → reload 后仍锁定）

## 变更概述

1. **#38 pan-undo 修复**：先写失败 ST 复现（undoable op → pan → Ctrl+Z 必须生效），定位根因（重点排查：pan 路径 wasm 闭包 panic 击穿 document keydown 监听器 / pointer capture 残留 / 焦点迁移至文本类目标），修复使撤销链路与视口 pan 完全解耦。合同：pan/zoom 纯视图操作，不进撤销栈、不破坏撤销栈、不使快捷键失效。
2. **#39 选中保持平移**（方案 A）：空白 pointerdown 不再立即清选；位移 ≥ 4px 视为 pan 拖动并保留全部选中；pointerup 时位移 < 4px（纯 click）才清选。图元命中分支不变；框选仍由 Shift+空白 / 框选工具触发（与 pan 手势天然区分）。只读模式同样保留选中。
3. **#40 标签字号**：新增「标签字号倍率」用户设置（localStorage `cdb.label-font-scale`，档位 0.8/1.0/1.25/1.5，默认 1.0，FloatingControls 循环按钮入口），与 zoom 解耦相乘作用于表名/字段名/注释；同时屏幕空间最小字号钳制——zoom 缩小时标签屏幕字号不低于 10px 下限（表名优先保证）。本机偏好，不写入图表文档。
4. **#41 区域锁定**：`Area.locked: bool`（前后端 DTO + DB 列 + diagram JSON `locked` 缺省 false）。入口：区域右键菜单项「锁定/解锁」+ Inspector 锁定复选框。锁定态：拖动/resize no-op（含多选整组拖动跳过锁定区域）、选中/Inspector/删除不受影响、边框角锁图标视觉提示。持久化走既有 PUT 快照通路；协作复用既有 area op（协议零变更）。


## 需要合并的 Delta 文件

### 1. deltas/database/coldrawdb-v1.sql

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/database/coldrawdb-v1.sql`
- 目标目录：`logos/resources/database/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/test/core-KB-shortcut-test-cases.md

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/test/core-KB-shortcut-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/fix-issues-38-41-canvas-interaction/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
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
   git add -A && git commit -m "docs(fix-issues-38-41-canvas-interaction): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issues-38-41-canvas-interaction`。
