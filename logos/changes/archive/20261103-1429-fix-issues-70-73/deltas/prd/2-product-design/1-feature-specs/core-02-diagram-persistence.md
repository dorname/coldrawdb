## MODIFIED — 5.1 导入流程

`POST /api/v1/diagrams/import` body 为 drawdb 导出的 JSON：

```json
{
  "title": "Imported Schema",
  "tables": [...],
  "references": [...],
  "areas": [...],
  "notes": [...]
}
```

导入始终创建独立新图，不复用调用方提供的全局实体 ID：

1. 保留现有逐实体规范化：缺 name 的表/字段及缺端点的关系单独丢弃并写入 warnings，缺失 ID 先自动补全，图名缺省为 `imported_diagram`。
2. 对每个保留下来的 table / field / reference / area / note 生成新的 `auto-` 全局唯一 ID；即使输入 ID 已存在或当前库中已有同 ID，也不原样写入。
3. 以 table / field 的旧新 ID 映射回填每条 reference 的 `start_table_id`、`end_table_id`、`start_field_id`、`end_field_id`；reference 自身也获得新 ID。字段归属、有效关系、表/字段/关系/区域/便签的数组顺序保持不变，既有图数据不被修改。
4. ID 隔离在共享 `persist_import_payload` 层实现，因此 diagram import 与 bridge import/retry 均避免跨图全局唯一冲突，不要求调用方先查询数据库或提交额外 remap 选项。
5. tables / references / areas / notes / dictionaries 任一整组 DTO 反序列化失败时返回 400，`details.kind=payload` 且 `details.message` 包含失败数组和 Serde 原因；不得退化为空数组并返回 `imported_tables=0` 的假成功。
6. 其他持久化失败保留既有空图回收，服务端 warning 日志记录 diagram_id 和错误但不记录 token/payload；500 响应 `details.kind=database`（或其他持久化类型）且 `details.message` 非空，不再吞掉底层错误。
7. 关系完整性检查前，将 `startTableId` / `endTableId` / `startFieldId` / `endFieldId` 分别补写到 `start_table_id` / `end_table_id` / `start_field_id` / `end_field_id`。snake_case 已有非空字符串时优先；其缺失、为空或类型无效时才采用 camelCase 值。补写后再执行现有缺端点丢弃与 ID 重映射。
8. `FieldDto.not_null` 同时接受 `notNull` 和 `not_null`；`ReferenceDto` 四个端点也声明对应 camelCase serde alias，保证直接 DTO 构造与规范化路径一致。
9. 两种命名风格导入后的关系数量、端点和字段 `not_null` 完全一致；两种键都缺失时保持既有 warnings 逐实体丢弃行为，不把无法解析的关系静默当作成功。
10. 上述旧→新 ID 映射只在导入规范化阶段生成并消费。导入完成后的关系写入口不得继续接受导入前的调用方旧 ID，也不得在房间首次物化或后续协作 op 中再次执行 remap；后续写入统一遵守 §5.4 的实际 ID 与端点归属约束。

## ADDED — 5.4 导入后关系写入口的一致性约束

导入规范化完成并持久化后，调用方必须以服务端返回或重新读取到的实体实际 ID 进行后续写入。MCP `update_reference` 的单个与批量 `action=create` 均适用以下约束：

1. `start_table_id`、`end_table_id`、`start_field_id`、`end_field_id` 必须是目标图当前快照中真实存在的 ID，不得使用导入前的旧 ID、其他图的实体 ID 或任意悬空 ID。
2. 除四个端点分别存在外，还必须校验 `start_field_id` 属于 `start_table_id`、`end_field_id` 属于 `end_table_id`；仅字段或表各自存在但归属不匹配，同样视为非法输入。
3. 单个与批量创建共用相同的端点存在性和字段归属校验。任一校验失败时，在 PUT 或 WS 写入前返回 `VALIDATION_ERROR`；批量创建任一项非法则整批不写入，不得先落库再依赖物化阶段修正。
4. 有效关系创建直接持久化调用方提供的实际端点，不生成或保存跨调用的旧→新 ID 映射，不将导入时的重键逻辑扩散到普通保存或协作写路径。
5. 协作房间首次物化只是加载当前持久化快照并应用 op，不是另一次导入；不得二次重写 table / field / reference / area / note ID 或关系四端点。
6. 本约束只阻止新增无效关系，不对既存悬空关系进行猜测性 remap、启发式修复、批量迁移或自动清理。历史数据无法可靠恢复旧→新映射时，保持原状并由后续独立变更处理。
