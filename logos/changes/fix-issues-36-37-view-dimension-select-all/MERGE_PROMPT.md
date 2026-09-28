# 合并指令

## 变更提案
- 提案名称：fix-issues-36-37-view-dimension-select-all
- 提案目录：logos/changes/fix-issues-36-37-view-dimension-select-all/

## 提案内容

# 变更提案：表/字段维度显式切换 + 画布全选与可撤销删除（#36 / #37）

> module: core | created: 2026-09-28

## 变更原因

来自 GitHub 两个 open issue：

- **#36 [Enhancement]**：当前「表维度」（拓扑档，仅表头）与「字段维度」（详情档）由缩放阈值（`zoom ≤ 0.5`）隐式切换，用户无法在两个维度内各自自由缩放；缩放语义被占用为视图模式开关。要求改为显式切换：右键菜单 + 左侧 ToolRail 按钮 + 快捷键三入口，维度模式独立持久（localStorage），缩放只负责缩放，UI 有状态指示。
- **#37 [BUG]**：画布焦点下 `Ctrl/Cmd+A` 触发浏览器整页全选而非全选画布图元；`Delete/Backspace` 无法删除选中的表（既有快捷键处理器仅覆盖关系/区域/便签，`SelectionKind::Table` 分支缺失；且既有 `on_delete_table` 绕过 CommandStack 不可撤销）。要求：画布焦点下 Ctrl/Cmd+A 全选图元并 preventDefault；Delete/Backspace 删除选中图元（含表，级联关系）且进撤销栈；输入框/Monaco/对话框内保留原生行为。

## 变更类型

设计级变更（交互/渲染合同修订 + 新键盘合同，含测试口径改写与代码实现）。

## 变更范围

- 影响的需求文档：无（S01 编辑器场景验收条件不含 LOD 触发口径与键盘细节，无需变更）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §5.12（LOD 分档：R-LOD-01 zoom 触发口径废止，改为显式维度模式；R-LOD-03/04 补偿改挂表维度；R-LOD-08 锚定措辞随档位来源调整）+ 新增 §5.13（画布键盘全选与删除合同）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` §15.4（`canvas.lod.tier1-max-zoom` 阈值 token 废止勘误；补偿 token 保留）
- 影响的业务场景：S01（编辑器画布，实现层）
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（非 API 变更）
- 影响的测试用例：
  - `logos/resources/test/core-CR-canvas-test-cases.md`：UT-CR-LOD-01 / ST-CR-LOD-01 改写为显式维度口径
  - `logos/resources/test/core-KB-shortcut-test-cases.md`：新增 UT-KB-05（Ctrl/Cmd+A 判定与门控纯函数）、UT-KB-06（DeleteTable 命令撤销/重做纯函数）、ST-KB-SEL-01（全选 + 删除 + 撤销 e2e）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 WASM 交互/渲染合同变更，verify 的 UT/ST/e2e 已覆盖验收口径；生产构建发布属既有独立管线，与先例（fix-31 / fix-34-35）一致
- 影响环境：无
- 是否涉及数据迁移：否（`cdb.view-dimension` 为新增 localStorage 视图偏好键，不落库 diagram 数据）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**#36 维度显式切换**：引入 `ViewDimension { Table, Field }` 视图偏好（默认 Field，localStorage `cdb.view-dimension` 持久，不落库）。渲染档位由显式维度决定：Table → 拓扑档渲染（仅表头 + 注释 + 表级锚定，沿用 #35 合同），Field → 详情档渲染；两者均可在任意 zoom 下保持。原 `lod_tier(zoom)` 阈值触发废止，zoom 仅驱动表维度内的字号/线宽可读性补偿（R-LOD-03/04 保留，改挂维度而非档位切换）。提供三入口：画布右键菜单切换项、ToolRail 维度按钮（激活态可见）、快捷键（`V`），并在 FloatingControls 增加状态指示（对齐「注释：…」按钮先例）。

**#37 全选与可撤销删除**：快捷键处理器新增两条合同——① `Ctrl/Cmd+A`（画布门控通过时）`preventDefault` 并全选画布图元（全部表 + 关系 + 便签 + 区域进入选中集，关系经两端表选中自动高亮）；② `Delete/Backspace` 增补 `SelectionKind::Table` 与多选/全选分支，删除表时级联其关系，统一经新增 `Command::DeleteTables { tables, references }`（批量快照，单表为长度 1 特例）进 CommandStack，单次 Undo 恢复整次删除；既有 `on_delete_table` 改为入栈路径。输入框 / contentEditable / Monaco / 对话框 / 命令面板 / 代码视图内沿用既有门控，不抢键。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-issues-36-37-view-dimension-select-all/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/fix-issues-36-37-view-dimension-select-all/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-issues-36-37-view-dimension-select-all/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-KB-shortcut-test-cases.md

- Delta 文件：`logos/changes/fix-issues-36-37-view-dimension-select-all/deltas/test/core-KB-shortcut-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issues-36-37-view-dimension-select-all): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issues-36-37-view-dimension-select-all`。
