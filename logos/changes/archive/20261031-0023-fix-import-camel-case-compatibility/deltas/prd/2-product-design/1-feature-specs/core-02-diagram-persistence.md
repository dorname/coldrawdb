## MODIFIED — 5.1 导入流程（追加命名兼容规则）

在现有逐实体容错、ID 重映射和错误诊断流程中增加以下规则：

1. 关系完整性检查前，将 `startTableId` / `endTableId` / `startFieldId` / `endFieldId` 分别补写到 `start_table_id` / `end_table_id` / `start_field_id` / `end_field_id`。snake_case 已有非空字符串时优先；其缺失、为空或类型无效时才采用 camelCase 值。补写后再执行现有缺端点丢弃与 ID 重映射。
2. `FieldDto.not_null` 同时接受 `notNull` 和 `not_null`；`ReferenceDto` 四个端点也声明对应 camelCase serde alias，保证直接 DTO 构造与规范化路径一致。
3. 两种命名风格导入后的关系数量、端点和字段 `not_null` 完全一致；两种键都缺失时保持既有 warnings 逐实体丢弃行为，不把无法解析的关系静默当作成功。

## MODIFIED — 5.2 字段映射（追加别名行）

| drawdb JSON | coldrawdb V1 | 说明 |
|---|---|---|
| `references[].startTableId / endTableId / startFieldId / endFieldId` | ✅ | 分别规范化为四个 snake_case 端点键；与既有 snake_case 等价 |
| `tables[].fields[].notNull / not_null` | ✅ | 统一反序列化为 `FieldDto.not_null` 并持久化 |
