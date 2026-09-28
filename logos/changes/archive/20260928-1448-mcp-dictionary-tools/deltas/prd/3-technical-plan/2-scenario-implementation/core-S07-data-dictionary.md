# Delta — core-S07-data-dictionary.md（mcp-dictionary-tools）

> 模块：core | 提案：mcp-dictionary-tools
> 目标：§10 废止「S06 不扩展 MCP 工具」口径，改为 MCP 正式支持数据字典管理。

## MODIFIED — 10. 与既有场景的关系

- **S01**：复用 debounce → PUT 快照与 revision/409 全链路；字典编辑只是快照内容新增字段
- **S02**：分享只读链路天然携带 dictionaries；Viewer 可浏览/导出、禁编辑
- **S05**：字典 op 复用 OT 通路，协议零变更（EX-7.5）
- **S06**：MCP 正式支持数据字典管理（mcp-dictionary-tools，2026-09-28）：
  - 新增 `update_dictionary` 工具（action: create/update/delete），语义与本场景对齐：
    - create 本地校验编码唯一（冲突拒绝，对齐 EX-7.1）
    - update 改 code 时同步改写所有引用字段的 `dict_code`（避免 EX-7.4 悬空绑定）
    - delete 级联置空所有引用字段（对齐 §5 级联置空语义），返回 `fields_cleared`
  - `update_field` 增补 `dict_code` 参数：空串解绑；非空编码本地校验字典存在，不存在即 `VALIDATION_ERROR` 拒绝（从源头杜绝悬空绑定）
  - 实现通路沿用 S06 既有「GET 全量 → 本地修改 → PUT 全量（expected_revision 乐观锁）」分片写入模式，无新增 HTTP 端点
