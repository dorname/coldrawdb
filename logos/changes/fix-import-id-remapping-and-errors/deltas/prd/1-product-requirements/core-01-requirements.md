## MODIFIED — 3. 功能性需求（Functional Requirements）

| FR ID | 需求 | 验收条件 | 对齐能力 |
|---|---|---|---|
| FR-01 | 支持表（含字段、主键、唯一、非空、自增、默认值、check、注释） | 创建 / 编辑 / 删除 / 移动；字段类型 7 引擎全覆盖 | CAP-CANVAS-01/02 |
| FR-02 | 支持关系（一对一/一对多/多对多；ON UPDATE/DELETE） | 起止字段可指定；关系类型正确生成 SQL | CAP-CANVAS-03 |
| FR-03 | 支持索引（单字段/复合/唯一） | 表内管理；导出 SQL 含索引 | CAP-CANVAS-04 |
| FR-04 | 支持区域（subject area 视觉分组） | 拖拽创建 / 编辑 / 删除 | CAP-CANVAS-05 |
| FR-05 | 支持便签（自由文本注释） | 富文本编辑；锚定到画布坐标 | CAP-CANVAS-06 |
| FR-06 | 撤销/重做（仅前端内存） | undo/redo 键可用；不持久化 | CAP-EDIT-01 |
| FR-07 | 编辑自动保存到 SQLite | debounce 1s；PUT 触发；失败有重试 | CAP-PERSIST-01 |
| FR-08 | 409 revision 冲突语义 | 携带 `expected_revision`；不匹配返回 409，前端提示用户 | CAP-PERSIST-02 |
| FR-09 | SQL 导出（7 引擎：MySQL / PostgreSQL / SQLite / MariaDB / MSSQL / OracleSQL / Generic） | 选择引擎 → 一键生成 SQL | CAP-BRIDGE-01 |
| FR-10 | SQL 导入（7 引擎） | 选择 SQL 文件 → 解析 → 渲染到画布 | CAP-BRIDGE-02 |
| FR-11 | DBML 导入/导出 | 双向支持 | CAP-BRIDGE-03 |
| FR-12 | JSON 导入/导出 | 通过 `POST /api/v1/diagrams/import`；同一 payload 重复导入创建实体 ID 隔离的独立新图并保留关系端点；嵌套类型错误返回可诊断 400，持久化失败返回非空 details，不得静默变成空图成功 | CAP-BRIDGE-04 / CAP-PERSIST-04 |
| FR-13 | 分享链接加载 | URL `?share=xxx` → 加载图 | CAP-PERSIST-03 |
| FR-14 | 校验 Issues | 重复表名 / 空字段名 / 字段类型合法 | CAP-EDIT-04 |
| FR-15 | 11 张表的数据模型 | diagrams / tables / fields / references / indices / areas / notes 等 | init.sql / database_design.json |
