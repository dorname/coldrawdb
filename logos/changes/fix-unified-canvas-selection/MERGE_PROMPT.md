# 合并指令

## 变更提案
- 提案名称：fix-unified-canvas-selection
- 提案目录：logos/changes/fix-unified-canvas-selection/

## 提案内容

# 变更提案：统一画布选择、拖动与平移交互

> module: core | created: 2026-10-31 | issue: #63

## 变更原因

Issue #63 指出画布工具语义不统一：默认「平移画布」实际同时负责选择，名称与行为不符；「选中关系」工具激活后点表完全无响应，用户必须反复切换工具才能完成基本的选择和拖动操作。拖表本身性能良好，问题在工具路由与可预期性，而非渲染性能。

本次以统一选择态为目标：选择类工具不因当前按钮而禁用表选择/拖动；关系工具只改变关系命中与创建能力，不应吞掉其他图元和空白手势。

## 变更类型

设计级变更（ToolRail 命名与画布指针路由调整，无 API / DB 变化）。

## 变更范围

- 影响的需求文档：无新增需求，沿用 S01 编辑器画布交互。
- 影响的功能规格：`core-01-editor-canvas.md` 新增统一选择合同；`core-01b-relationship.md` §3.1 修订选中关系工具；`core-04-side-panel-tabs.md` ToolRail 按钮命名。
- 影响的业务场景：S01 编辑器画布 / 图元选择与拖动。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的编排测试：无独立 API 编排变化。
- 影响的测试：`core-PB-relationship-test-cases.md` 新增路由单测与真实浏览器交互用例；既有 ST-CR-02、ST-CR-MULTI-01、ST-CR-PAN-02、ST-PB-16 保持回归。
- 不影响：创建关系工具的专用手势、框选多选语义、只读权限、撤销/保存/协作协议。

## 部署影响

- 是否需要部署：否。
- 部署原因：纯前端交互与测试调整，无配置、API、DB 或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

将默认工具提示由「平移画布」改为「选择 / 平移」，明确表头/表体负责选择拖动、空白负责平移。选中关系工具先按既有精度命中并选择关系；未命中关系时不再提前返回，而是回落到统一的表/便签/区域选择与空白平移通路，同时禁止字段连接点在该工具下发起建关系。

新增纯函数路由合同与真实浏览器用例，覆盖关系选择工具下点表、拖表、点线、空白平移及不误建关系；默认拖表、框选、多选、只读与既有关系创建行为继续由原有用例回归。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-unified-canvas-selection/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-unified-canvas-selection/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/fix-unified-canvas-selection/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-unified-canvas-selection/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-unified-canvas-selection): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-unified-canvas-selection`。
