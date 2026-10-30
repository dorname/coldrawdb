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
