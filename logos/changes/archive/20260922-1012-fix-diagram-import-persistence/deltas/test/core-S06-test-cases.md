# Delta — core-S06-test-cases.md（修改）

> 模块：core | 提案：fix-diagram-import-persistence

## ADDED — UT-MCP-28 — import_schema payload 前置规范化

- **位置**：`mcp-server/src/service.rs`（`import_schema`）
- **GIVEN**：payload 无顶层 `name`，表/字段无 `id`（如 AI 手工构造的 drawdb JSON）
- **WHEN**：调用 `import_schema`
- **THEN**：发往 POST /api/v1/diagrams/import 的 body 中：payload.name 缺省为 `"imported_diagram"`；每张缺 id 的表获得 `auto-` 前缀全局唯一 id；每个缺 id 的字段同样补 id；已有 name/id 原样保留
- **目的**：即使后端未升级（旧版静默丢表），adapter 规范化后也能正确持久化——修复对只换 adapter 的场景同样生效

## ADDED — UT-MCP-29 — Diagram.name 可空契约（存量脏图防御）

- **位置**：`logos/resources/api/mcp-tools.yaml`（经 `include_str!` 进二进制）
- **GIVEN**：tools/list 暴露的 `Diagram` outputSchema
- **WHEN**：检查 `name` 属性
- **THEN**：`name` 类型为 `["string","null"]`（键仍在 required 中）；对 mock 的 `name: null` 图执行 get_diagram 后，structuredContent 形状为 `{diagram: {...}}` 且 diagram.name 为 null 时不再违反契约
- **目的**：修复前导入产生的存量脏图不再导致客户端 `Output validation error` 硬失败

## ADDED — ST-MCP-14 — stdio 端到端：缺 id payload 导入后结构校验

- **GIVEN**：mock 后端；payload 为 `{tables:[{name,x,y,fields:[{name,type}]}]}`（表/字段均缺 id，无 name）
- **WHEN**：stdio → `import_schema` → `get_diagram(diagram_id)`
- **THEN**：import 响应 `imported_tables == 1`；get 响应 `diagram.tables[0].id` 非空（auto- 前缀）、`diagram.tables[0].fields[0].id` 非空、`diagram.name` 为 `"imported_diagram"`（或 payload.name）；`structuredContent` 通过各自 outputSchema 校验
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S06"`）

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、ST-MCP-03/14 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、ST-MCP-04/05 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、ST-MCP-06～09 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27 |

### 用例登记（OpenLogos verify 解析用）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-23 | mock 上游 PUT 200 且 `data` 含额外字段 | update_* 四工具恰好 `{id,revision}`；`layout_diagram` 恰好 `{id,revision,tables_repositioned}`；缺 `id` 以路径补齐；`revision` 缺/非数→`UPSTREAM_INVALID` |
| UT-MCP-24 | `update_reference` 带 / 不带 `color` | create 含 `color`；update 不传保持原值；`color` >64 → `VALIDATION_ERROR` |
| UT-MCP-25 | 上游 409 `details.code=USE_OP_CHANNEL` | 映射 `USE_OP_CHANNEL`，不是 `REVISION_CONFLICT` |
| UT-MCP-26 | 房间 diff + 假 socket 提交 op | `table.update` 不含 fields；ack 后 revision 为 serverRev；READ_ONLY / 空 diff / 无 token 符合断言 |
| UT-MCP-27 | tool_call 成功与失败 | 成功不产生 stderr 行；失败 JSON 含 code 与 message |
| UT-MCP-28 | import payload 缺 name/表字段缺 id | POST body 补默认 name；表/字段补 auto- id；已有值保留 |
| UT-MCP-29 | tools/list 的 Diagram schema | `name` 类型为 `["string","null"]`；`name:null` 图不再违反契约 |
| ST-MCP-13 | stdio → `layout_diagram(mode=force)` | `structuredContent` 过 `outputSchema`；`content[0].text` == `structuredContent`；坐标更新且 revision 递增 |
| ST-MCP-14 | stdio → import（缺 id payload）→ get | 表/字段 id 已补全；name 兜底；structuredContent 过 outputSchema |
