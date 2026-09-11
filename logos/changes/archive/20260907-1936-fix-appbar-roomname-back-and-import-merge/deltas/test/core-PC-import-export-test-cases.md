# Delta: core-PC-import-export-test-cases.md

> 提案：fix-appbar-roomname-back-and-import-merge

## MODIFIED — ST-PC-01 用例行（主表）

| ST-PC-01 | 编辑器已加载（已有 2 表） | 更多菜单 → 导入 → 粘贴含 2 张表的 DDL → 提交 | 解析摘要可见；**合并进当前画布**（表数 2→4，新增表避让现有布局）；落账 PUT 含 4 表；Toast「已导入」；无页面跳转（URL 不变）；不调用 `/bridge/import/local` |

## ADDED — UT-PC-09 用例行（主表）

| UT-PC-09 | store 已有表（含坐标）+ 导入解析结果（tables + references） | `merge_import_into_store(store, tables, references)` | 返回合并后的表/关系；新表 x/y 避让现有表包围盒（网格落位不重叠）；既有表坐标不变；references 全量追加 |

## ADDED — UT-PC-09 — 导入合并纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`merge_import_into_store`）
- **步骤与预期**：
  1. 现有 1 表 (x=100,y=100)，导入 2 表 → 新表落在现有内容右侧/下方网格，不与现有表包围盒重叠；现有表坐标不变
  2. 导入 references 追加到合并结果末尾，端点 id 原样保留（解析时已解析到导入表实体 id）
  3. 空 store（0 表）→ 新表从默认起点网格落位
  4. 幂等性由调用方保证（纯函数不修改入参，返回新 Vec）

## MODIFIED — 变更记录

| ST-PC-01 / UT-PC-09（relation-inspector-and-ddl-io 后续修正） | MODIFIED/ADDED | 导入语义由「bridge 新建游离 diagram + 跳转」改为「本地合并进当前画布」；不再断言 bridge 返回 diagramId |
