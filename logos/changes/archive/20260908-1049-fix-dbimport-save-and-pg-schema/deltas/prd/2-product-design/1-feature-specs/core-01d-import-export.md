# Delta: core-01d-import-export.md

> 提案：fix-dbimport-save-and-pg-schema

## MODIFIED — §4.1 布局（数据库 Tab）

数据库 Tab 区块改为：

```
数据库 Tab：
┌──────────────────────────────────────┐
│ 数据库引擎: [ sqlite ▼ ]             │  ← import-db-engine（sqlite/postgres）
│ 连接信息: [___________________]      │  ← import-db-source
│ Schema:     [ public _______ ]      │  ← import-db-schema（仅引擎=postgres 时显示）
│ [ 连接并解析 ]                        │  ← import-db-connect
└──────────────────────────────────────┘
```

- 数据库来源 testid 清单追加：`import-db-schema`。
- `import-db-schema` 仅在引擎选择 `postgres` 时渲染；占位与缺省值均为 `public`，留空按 `public` 发送；引擎为 `sqlite` 时不渲染且请求不携带 `schema`。

## MODIFIED — §4.4 提交行为（数据库连接导入流程）

3.5 数据库连接导入流程补充：

- 引擎为 `postgres` 时请求体携带 `schema`（输入框值，去空白；为空则 `public`）；引擎为 `sqlite` 时不携带。

## ADDED — §4.4 提交行为（导入实体 ID 重键条款）

新增条款（fix-dbimport-save-and-pg-schema）：

- **导入实体 ID 重键**：`merge_import_into_store` 在合并前对导入产生的新表/字段/关系统一做 ID 重键——以全局唯一 ID 生成器（`new_entity_id`，`{prefix}-{16位hex}`）替换解析期确定性 ID（`import-t-{i}` / `import-t-{i}-f{n}` / `import-r-{n}`），并按 旧→新 映射同步改写关系（reference）的 start/end table/field 四个端点。
- 原因：后端 `table`/`field`/`reference` 的 `id` 为全局单列主键，确定性 ID 在任意第二次导入保存时必然主键冲突（500）。重键后任意次数、任意 diagram 的导入保存均不冲突。
- 口径：重键只发生在合并入口一次；`parse_*_import_tables` 解析器内部 ID 形式不变（纯函数行为保持可测）。
