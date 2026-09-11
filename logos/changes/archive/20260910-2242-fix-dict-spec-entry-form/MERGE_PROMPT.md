# 合并指令

## 变更提案
- 提案名称：fix-dict-spec-entry-form
- 提案目录：logos/changes/fix-dict-spec-entry-form/

## 提案内容

# 变更提案：fix-dict-spec-entry-form

> module: core | created: 2026-09-10

## 变更原因
S07（feat-data-dictionary，已归档）实现时发现：生产前端 `LeftPanel`/`SidePanelTab` 侧栏 Tab 体系是死代码（从未实例化），线上布局为 AppBar + ToolRail + Canvas + Inspector 抽屉。字典入口因此落地为 **ToolRail `toolrail-dicts` 按钮 + DictPanel 右侧抽屉**（与主原型 `core-01-editor-prototype.html` 的 testid 完全一致），但规格文字仍写「侧栏新增字典 Tab」，规格与实现/原型不一致。本变更仅做规格文字口径对齐，不改任何代码。

## 变更类型
设计级（文档口径修正——实现与原型已是目标形态，无代码任务）

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：
  - `core-01e-data-dictionary.md` — §2 标题与入口描述、§2.2 操作表（双击/右键未实现，实际为详情内联 blur 编辑）、§5「字典 Tab」措辞
  - `core-04-side-panel-tabs.md` — §7A 标题（「V1 仅前端 state」亦过期：字典已经 migration 0007 持久化）与内容、§10.5 V1 边界注、§11 UT-SP-11 描述
- 影响的业务场景：S07（仅测试用例文件措辞）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（`core-S07-test-cases.md` 仅 ST-S07-02/03 两行「字典 Tab」措辞对齐，用例 ID 与断言不变）
- 影响的 smoke 测试：无

## 部署影响
- 是否需要部署：否
- 部署原因：纯规格文档文字修正，无代码/配置变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
将 `core-01e` 与 `core-04` 中数据字典的入口形态描述从「侧栏 Tab（core-04 体系）」对齐为实际落地形态「ToolRail `toolrail-dicts` 按钮 + DictPanel 抽屉（testid `dict-panel`）」，并注明生产前端侧栏 Tab 体系未上线（LeftPanel 为死区）、入口形态以主原型为准。同小节内一并修正两处过期描述：§2.2/§7A.2 中未实现的「双击重命名 / 右键上下文菜单」改为实际的详情内联 blur 编辑口径；§7A 标题删除「V1 仅前端 state」（字典已随 migration 0007 持久化，与 Enum/Types 口径不同）。`core-S07-test-cases.md` 两行「字典 Tab」措辞同步为「字典面板」，用例 ID 与断言口径不变。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md

- Delta 文件：`logos/changes/fix-dict-spec-entry-form/deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/fix-dict-spec-entry-form/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-S07-test-cases.md

- Delta 文件：`logos/changes/fix-dict-spec-entry-form/deltas/test/core-S07-test-cases.md`
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
   git add -A && git commit -m "docs(fix-dict-spec-entry-form): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-dict-spec-entry-form`。
