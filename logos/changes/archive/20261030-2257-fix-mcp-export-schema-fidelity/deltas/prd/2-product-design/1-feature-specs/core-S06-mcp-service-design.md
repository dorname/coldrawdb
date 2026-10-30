## MODIFIED — 4.1 读调用

- `list_diagrams` 默认最多返回 100 条摘要；adapter 负责稳定排序和本地 `limit` 截断。
- `get_diagram` 返回完整结构化 JSON，同时提供简短 text 摘要，避免客户端只能解析自然语言。
- `export_schema` 返回 `{diagram_id, revision, format, mime_type, content}`；相同输入和 revision 必须字节一致。表、字段、主键列和关系均按 diagram 数组原顺序输出，不依赖哈希表遍历顺序；JSON 的既有 pretty 序列化和所有 MIME 类型保持不变。
- SQL 导出对七种格式统一遵守以下 schema 保真规则：
  - 单字段主键在该列内输出 `PRIMARY KEY`；两个及以上字段组成的主键不在任何列内重复声明，而是在字段清单之后输出唯一表级 `PRIMARY KEY (column_a, column_b, ...)`，列顺序与字段数组中主键字段的顺序一致。
  - 字段 `not_null=true`、`unique=true` 分别输出 `NOT NULL`、`UNIQUE`；非空 `default` 作为 SQL 表达式原样输出为 `DEFAULT <expression>`，不擅自加引号或求值。
  - 对每条 `reference`，以 `start_table_id/start_field_id` 为本表外键列，以 `end_table_id/end_field_id` 为被引用表和列，在 start 表的字段及主键定义之后输出 `FOREIGN KEY (...) REFERENCES ... (...)`。非空 `on_delete`、`on_update` 转为大写并把下划线转为空格，分别追加 `ON DELETE <action>`、`ON UPDATE <action>`；缺省动作不追加。
  - PostgreSQL、Oracle、generic 在对应 `CREATE TABLE` 后输出 `COMMENT ON TABLE ... IS ...` 和 `COMMENT ON COLUMN ... IS ...`；MySQL、MariaDB 分别使用字段行内 `COMMENT '...'` 和表选项 `COMMENT='...'`；SQLite、MSSQL 本期省略表和字段注释，不生成不兼容的 `COMMENT ON`，也不扩展 MSSQL extended property。
  - SQL 标识符沿用各方言现有引用规则；注释按 SQL 字符串字面量转义，空注释不生成语句。
- DBML 导出遵守以下规则：
  - 单字段主键使用列内 `[pk]`；复合主键不在列内标 `pk`，改在对应 Table 内输出 `indexes { (column_a, column_b, ...) [pk] }`。
  - 字段设置按 `pk`（仅单主键）、`not null`、`unique`、`default`、`note` 的稳定顺序组合；非空 `default` 保留原表达式语义，数字、布尔和 NULL 直接输出，其余值使用 DBML 反引号表达式并转义反斜杠和反引号。
  - 非空字段 `comment` 输出字段 `note`，非空表 `comment` 输出 Table 的 `Note`；note 按 DBML 字符串语法转义。
  - 有效关系输出 `Ref: start_table.start_field > end_table.end_field`；非空 `on_delete`、`on_update` 转为小写并把下划线转为空格，写入 `[delete: <action>, update: <action>]`，只存在一个动作时仅输出该动作。
- SQL 与 DBML 均跳过任一表或字段端点无法解析的悬空关系，不得输出 `?`、空标识符或伪造约束；跳过不影响其他有效关系和确定性。
- 导出仍为 adapter 内纯函数，不引入新依赖，不访问数据库、网络或文件系统。
