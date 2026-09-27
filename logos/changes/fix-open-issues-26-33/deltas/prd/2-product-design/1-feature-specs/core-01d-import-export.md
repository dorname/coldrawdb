# Delta — core-01d-import-export.md（fix-open-issues-26-33 / #29）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：数据库导入路径合并 references，消除两条导入路径的结果不对称。

## ADDED — 4.8 数据库导入关系线合并与双路径对称验收口径

> 提案：fix-open-issues-26-33（GitHub issue #29）

### 4.8.1 数据库导入（import/connect）前端消费

- 后端响应 `data` 自本变更起恒含 `references` 键（名址数组，见 `core-03-bridge-io.md` §13.1/§13.3）。
- 前端轻适配层 `parse_bridge_import_tables(tables, references)` 升级：
  1. `tables` 仍复用 JSON 导入解析（确定性 ID `import-j-t{i}` / `{tid}-f{j}` 不变，UT-PC-20 重键口径不受影响）；
  2. `references` 按 **表名 + 列名** 解析到上述确定性表/字段 ID，产出画布 `Reference`（`start_table_id/start_field_id` = FK 所在表/列，`end_*` = 被引用表/列；`type_` 由 on_delete/on_update 映射，同 SQL 导入口径；`color`/`line_type`/`stroke_style` 缺省空串）；
  3. 名址无法解析（表/列不存在）的条目跳过，不报错、不产悬空引用；
  4. 解析结果连同 references 一起进入 `merge_import_into_store(store, tables, references)`，与 SQL 导入同一路径落画布。
- 旧调用形参（仅 `tables`）由调用点一并迁移；`references` 缺省（老后端/异常响应）按空数组处理，行为回退为「仅表结构」，不报错。

### 4.8.2 双路径对称验收口径

同一数据源（同一 PG schema）分别经两条路径导入后，画布语义应接近一致：

| 口径 | SQL 导入（dump 文本） | 数据库导入（import/connect） |
|---|---|---|
| 表数 | 相等 | 相等 |
| 关系数 | 相等（FK → references） | 相等（fks → references） |
| 非空表/列 comment 数 | 接近一致（dump 含 `COMMENT ON` / 行内 `COMMENT` 时） | 一致（obj_description / col_description 直采） |

允许差异：实体 ID、坐标/布局、以及 dump 未包含 `COMMENT ON` 语句时 SQL 路径注释数少于数据库路径（数据源本身不含的信息不要求凭空对齐；此时以「数据库路径为注释真值」提示用户优先使用数据库导入）。

### 4.8.3 只读与边界

- 导入合并后 references 与表一样进入 dirty → 保存/协作 OT 既有路径，无新增持久化通道。
- introspect 仍不含视图/索引/触发器/序列（`core-03-bridge-io.md` §13.2），references 仅来自 FK 约束。
