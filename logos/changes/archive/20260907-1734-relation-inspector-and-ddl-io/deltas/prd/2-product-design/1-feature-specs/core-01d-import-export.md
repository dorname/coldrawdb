# Delta: core-01d-import-export.md

## MODIFIED — 4.4 提交行为

1. 校验：内容非空；SQL 需选引擎；JSON 需合法
2. **SQL 导入解析（relation-inspector-and-ddl-io：列级 DDL 解析，替换原「每表硬造 id 字段」最小桩）**——`parse_sql_import_tables` 口径：
   - 列定义：列名、类型整串原样保留（含 `VARCHAR(32)` / `DECIMAL(10,2)` 参数化）、列级约束 `PRIMARY KEY` / `NOT NULL` / `UNIQUE` / `AUTO_INCREMENT`（PG `SERIAL`/`BIGSERIAL` 类型 → `increment=true`）/ `DEFAULT <值>` / `COMMENT '...'`
   - 表级约束：`PRIMARY KEY(a[, b])`（含联合主键）、`FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `c ... REFERENCES t(c2)` → 生成 `references` 连线（cardinality 按 core-01b §2 推导口径：目标列唯一/主键 → one_to_one，否则 one_to_many）
   - 标识符规范化：反引号 / 双引号 / 方括号去除；未知类型整串原样保留（不做方言映射）
   - 非 CREATE TABLE 语句跳过；无合法 CREATE TABLE → 报错「未检测到数据表」
   - **V1 不做**：CHECK、INDEX/KEY 索引定义、ALTER TABLE 序列、视图/触发器、引擎方言全量映射
3. `POST /api/v1/bridge/import/local` body：`{ format, content, engine?, title? }`（payload 由解析结果组装，API 契约不变）
4. 成功（`status: success` 或返回 `diagramId`）→ `window.location` 跳转 `/editor/{diagramId}`
5. 失败 → 抽屉内 inline 错误 + ErrorToast
6. 按钮文案：**导入并打开**；提交中 disabled + `导入中...`

## MODIFIED — 5.2 预览生成（客户端）

| format | 函数 | V1 范围 |
|--------|------|---------|
| SQL | `export_diagram_sql(store, engine)` | `CREATE TABLE` + 列定义（类型整串 + PK/NOT NULL/**UNIQUE**/**DEFAULT**/**自增**（引擎映射：mysql/generic → `AUTO_INCREMENT`；postgresql 由 `SERIAL` 类型承担不追加）/**COMMENT**：mysql 行内 `COMMENT 'x'`，其余引擎后置 `COMMENT ON COLUMN t.c IS 'x'`）+ FK 引用 `references`（表末 `FOREIGN KEY (c) REFERENCES t(c2)`） |
| DBML | `export_diagram_dbml(store)` | 表 + 字段 + `ref:` 关系 |
| JSON | `serde_json::to_string_pretty(diagram)` | 与 persistence JSON 同构 |

空 diagram：预览区显示「暂无表，无法导出」；复制/下载 disabled。
