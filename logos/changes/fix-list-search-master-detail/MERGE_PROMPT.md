# 合并指令

## 变更提案
- 提案名称：fix-list-search-master-detail
- 提案目录：logos/changes/fix-list-search-master-detail/

## 提案内容

# 变更提案：修复列表搜索与字段网格状态一致性

> module: core | created: 2026-10-31 | issue: #64

## 变更原因

Issue #64 指出列表视图搜索后，左树按关键字过滤，右侧却仍显示搜索前选中的表；无命中时左树为空、右网格仍旧表，造成“无结果”和详情并存的不一致。搜索框写“搜索表…”，实际复用的 `filter_tables` 同时匹配表名、字段名和首字段类型，字段命中又缺少原因提示。

旧规格与测试把“过滤不影响右网格当前表”当成预期，需要随本次 Bug 修复一并调整，不能只改断言掩盖问题。

## 变更类型

设计级变更（ListView master-detail 搜索行为与提示修复，无 API/DB 变化）。

## 变更范围

- 影响的功能规格：`core-04-side-panel-tabs.md` §10.5；主原型 `core-01-editor-prototype.html` 的 ListView。
- 影响的业务场景：S01 列表视图浏览与字段编辑。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的测试：`core-SP-side-panel-test-cases.md` ST-SP-LIST-03，新增 ST-SP-LIST-04；`core-UI-modals-2-test-cases.md` UT-MM-38，新增 UT-MM-39。
- 不影响：`filter_tables` 在 Tables Tab 的过滤口径、树分组/折叠、字典选中、字段编辑与落账、清空搜索后的原选中表。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos 前端行为、主原型和 Playwright 回归，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

ListView 当前表解析改为基于搜索后的可见表集合：当前表仍在结果中则保持，不在结果中则回落首个匹配表，无匹配则返回 None。右侧网格在无匹配时显示专用空态并隐藏旧表标题、关联摘要和字段表头；清空搜索后仍按既有选中规则恢复，不修改 diagram 数据。

搜索框文案明确为“按表名/字段名/类型搜索…”。非表名命中的树节点显示具体字段名或类型命中原因。同步更新唯一主原型和旧测试中的相反预期，新增纯函数与真实 Playwright 回归。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/fix-list-search-master-detail/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-list-search-master-detail/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/fix-list-search-master-detail/deltas/test/core-SP-side-panel-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-UI-modals-2-test-cases.md

- Delta 文件：`logos/changes/fix-list-search-master-detail/deltas/test/core-UI-modals-2-test-cases.md`
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
   git add -A && git commit -m "docs(fix-list-search-master-detail): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-list-search-master-detail`。
