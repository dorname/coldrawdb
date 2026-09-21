# core-S06 MCP 服务测试用例

> module: core | scenario: S06 | proposal: align-unified-prototype-and-add-mcp

## 1. 测试边界

MCP 测试分三层：纯函数/配置 UT、mock HTTP 的 MCP stdio ST、与真实隔离 backend 的 orchestration。所有自动化用例必须通过 OpenLogos reporter 追加到 `logos/resources/verify/test-results.jsonl`；日志与结果中不得出现测试 Token 明文。

## 2. UT 用例

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-01 | 缺失或非法 `COLDRAWDB_BASE_URL` | 启动失败，错误 `CONFIG_INVALID`，退出码 2 |
| UT-MCP-02 | initialize | 返回 serverInfo、tools capability、instructions；协议版本协商正确 |
| UT-MCP-03 | tools/list | 恰好七个工具；名称、inputSchema、outputSchema 稳定 |
| UT-MCP-04 | list 响应规范化 + query/limit | 过滤、排序、截断正确，不泄漏上游 envelope |
| UT-MCP-05 | get diagram 响应 | 保留完整模型和 revision，structuredContent 符合 schema |
| UT-MCP-06 | JSON/DBML/七类 SQL 导出 | 相同 diagram 输出确定；无数据库或网络副作用 |
| UT-MCP-07 | create 请求 | 只发送 name/database；解析 `{code,data,request_id}` |
| UT-MCP-08 | update 请求 | 显式发送 expected_revision 和完整 diagram；成功 revision +1 |
| UT-MCP-09 | delete confirm=false/缺失 | 本地拒绝；不发送 HTTP；confirm=true 才允许 |
| UT-MCP-10 | import payload | 非 object 本地拒绝；drawdb_json object 正确映射 |
| UT-MCP-11 | 四客户端配置 fixture | Claude/Cursor JSON、Codex TOML、OpenCode JSON 可解析且命令/环境一致 |
| UT-MCP-12 | 上游 409 | 映射 `REVISION_CONFLICT`，保留 current_revision，retryable=false |
| UT-MCP-13 | 400/401/403/404/422/5xx | 映射表一致；只暴露白名单 details/request_id |
| UT-MCP-14 | 连接失败/超时 | 分别映射 unavailable/timeout；写操作不自动重试 |
| UT-MCP-15 | Token 和 stdout | stdout 只有协议帧；stderr、错误、Debug、reporter 无 Token 明文 |
| UT-MCP-16 | update_table 参数校验 | 缺少 table_id → `VALIDATION_ERROR`；name 超长 → `VALIDATION_ERROR`；x/y 非 number → `VALIDATION_ERROR` |
| UT-MCP-17 | update_field 参数校验 | 缺少 field_id → `VALIDATION_ERROR`；name 超长 → `VALIDATION_ERROR`；type 非法 → `VALIDATION_ERROR` |
| UT-MCP-18 | update_reference 参数校验 | action=create 但缺少 start/end_table_id/field_id → `VALIDATION_ERROR`；action=update/delete 但缺少 ref_id → `VALIDATION_ERROR`；cardinality 非法值 → `VALIDATION_ERROR` |
| UT-MCP-19 | layout_diagram 力导向算法 | 3 表 2 边输入 → 输出不重叠（最小间距约束满足）；相同输入两次运行 → 输出完全一致（确定性）；孤立表（无边）→ 位置不变 |
| UT-MCP-20 | tools/list 工具总数 | 恰好 11 个工具；新增 4 个工具 name/title/description/inputSchema/outputSchema 稳定 |
| UT-MCP-21 | update_table 端到端（mock HTTP） | GET → 修改 name/comment → PUT；PUT body 中仅该表属性变更，其他表不变 |
| UT-MCP-22 | update_reference create/delete（mock HTTP） | create：references 数组新增一条边；delete：指定 ref_id 从 references 中移除；revision 均 +1 |

## 3. ST 用例

| ID | 前置与步骤 | 预期 |
|---|---|---|
| ST-MCP-01 | 启动 stdio → initialize → initialized → tools/list | 握手成功，七工具可见，annotations 正确 |
| ST-MCP-02 | list → get → export JSON/DBML/SQL | id/revision 一致，导出内容非空且可重复 |
| ST-MCP-03 | create → get → update → import → delete | 主链全部成功；revision 从 0 变 1；删除需 confirm |
| ST-MCP-04 | 两次使用同一 expected_revision 更新 | 第一次成功，第二次 `REVISION_CONFLICT`，数据未覆盖 |
| ST-MCP-05 | mock 401/403/404/422/500/timeout + 注入测试 Token | 错误码稳定；所有输出完成脱敏 |
| ST-MCP-06 | Claude `.mcp.json` 启动并 tools/list | 配置解析和 stdio 握手通过 |
| ST-MCP-07 | Codex `config.toml` 启动并 tools/list | `default_tools_approval_mode="writes"` 生效或配置结构校验通过 |
| ST-MCP-08 | Cursor `.cursor/mcp.json` 启动并 tools/list | 配置解析和 stdio 握手通过 |
| ST-MCP-09 | OpenCode `opencode.json` 启动并 tools/list | local command/environment 解析和 stdio 握手通过 |
| ST-MCP-10 | 启动 stdio → get_diagram → update_table(name/comment) → update_field(name/type) → get_diagram 验证 | 表名/表注释/字段名/字段类型已更新；revision 递增；其他表和字段未被修改 |
| ST-MCP-11 | 启动 stdio → get_diagram → update_reference(create) → update_reference(update cardinality) → update_reference(delete) → get_diagram 验证 | 连线创建/更新/删除均生效；revision 递增；references 数组最终为空或符合预期 |
| ST-MCP-12 | 启动 stdio → get_diagram → layout_diagram(mode=force) → get_diagram 验证 → layout_diagram(mode=force) 再次执行 | 两次 layout_diagram 输出表位置完全一致（确定性）；表间无重叠；revision 递增 |

> CI 若没有某个客户端二进制，ST-MCP-06～09 可由配置解析 + 官方 schema fixture 执行；至少 ST-MCP-01 必须使用真实 MCP client/Inspector 完成协议握手，不能全部以静态检查代替。

## 4. 验收标准追溯

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

## 5. Reporter 契约

每个测试完成后写一行 JSON：

```json
{"test_id":"UT-MCP-01","status":"PASS","duration_ms":12,"timestamp":"2026-08-18T00:00:00Z","module":"core","scenario":"S06"}
```

- `test_id` 必须在本文件登记且唯一。
- 失败写 `FAIL` 并可带脱敏 `message`，不得写 Token 或完整 diagram payload。
- 编排 runner 失败时也必须报告已执行步骤；不能因进程退出丢失整个结果账本。

---

## 合并自 fix-remote-github-issues-7-18（2026-09-18）

## ADDED — UT-MCP-23 — 写工具瘦响应 normalize（契约）

- **位置**：`mcp-server/src/service.rs`（`update_diagram` / `update_table` / `update_field` / `update_reference` / `layout_diagram`）
- **GIVEN**：mock 上游 PUT 200，`data` 为**含额外字段的形状**（如整图 diagram 或 `{id, revision, extra...}`）
- **WHEN**：调用各写工具
- **THEN**：返回值经各自 `outputSchema` 严格校验通过：
  - update_* 四工具：恰好 `{ id, revision }`，无 envelope / 无整图字段
  - `layout_diagram`：恰好 `{ id, revision, tables_repositioned }`
  - 上游 `data` 缺 `id` 时以请求路径 id 补齐；`revision` 缺/非数 → 映射 `UPSTREAM_INVALID`（或规格错误码），不得产出缺字段的成功响应

## ADDED — UT-MCP-24 — update_reference 支持 color 参数

- **断言**：`action=create` + `color:"#3af"` → PUT 的 diagram 中该 reference 含 `color`；`action=update` 不传 `color` → 原值保持；`color` 超长（>64）→ `VALIDATION_ERROR`

## ADDED — ST-MCP-13 — stdio 端到端：layout_diagram structuredContent 过校验

- **GIVEN**：mock 后端含多表+关系图；PUT 返回 `{id, revision}` 瘦响应
- **WHEN**：stdio → `layout_diagram(mode=force)` → 读取 tools/call 结果
- **THEN**：`structuredContent` 通过 `layout_diagram.outputSchema` 校验；`content[0].text` 反序列化 == `structuredContent`；坐标已更新且 revision 递增；同环境不再需要 curl 手工写回
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

### 用例登记（OpenLogos verify 解析用）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-23 | mock 上游 PUT 200 且 `data` 含额外字段 | update_* 四工具恰好 `{id,revision}`；`layout_diagram` 恰好 `{id,revision,tables_repositioned}`；缺 `id` 以路径补齐；`revision` 缺/非数→`UPSTREAM_INVALID` |
| UT-MCP-24 | `update_reference` 带 / 不带 `color` | create 含 `color`；update 不传保持原值；`color` >64 → `VALIDATION_ERROR` |
| ST-MCP-13 | stdio → `layout_diagram(mode=force)` | `structuredContent` 过 `outputSchema`；`content[0].text` == `structuredContent`；坐标更新且 revision 递增 |
| ST-MCP-14 | stdio → import（缺 id payload）→ get | 表/字段 id 已补全；name 兜底；structuredContent 过 outputSchema |

## UT-MCP-25 — USE_OP_CHANNEL 不映射为 REVISION_CONFLICT

- **位置**：`mcp-server/src/error.rs`
- **GIVEN**：上游 HTTP 409，body.details.code 为 `USE_OP_CHANNEL`
- **WHEN**：映射为 ToolError
- **THEN**：`code` 为 `USE_OP_CHANNEL`，不是 `REVISION_CONFLICT`；`details.code` 保留。普通 409（仅 `current_revision`）仍为 `REVISION_CONFLICT`

## UT-MCP-26 — 房间图经 op 通道写入

- **位置**：`mcp-server` 协作 diff / 会话逻辑
- **GIVEN**：活跃房间 `diagramId` 匹配；基准表名与目标表名不同；假 socket 先回 `connected` 再回 `ack.serverRev`
- **WHEN**：规划并提交房间写入
- **THEN**：产生不含 fields 的 `table.update`；提交后 revision 为 ack 的 serverRev；`READ_ONLY` 在第一帧失败且不再继续发送；空 diff 保持当前 revision；无 token 为 `UNAUTHENTICATED` 且错误文本不含 token

## UT-MCP-27 — 成功调用不写 stderr

- **位置**：`mcp-server` tool_call 日志
- **GIVEN**：tool_call 结果为成功或失败
- **WHEN**：决定是否写 stderr
- **THEN**：`status=ok` 不产生日志行；`status=error` 的 JSON 含 `code` 与 `message`，不含 diagram 正文

## 合并自 fix-diagram-import-persistence（2026-09-21）

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
- **THEN**：`name` 类型为 `["string","null"]`（键仍在 required 中）；对 `name: null` 图执行 get_diagram 后 structuredContent 不再违反契约
- **目的**：修复前导入产生的存量脏图不再导致客户端 `Output validation error` 硬失败

## ADDED — ST-MCP-14 — stdio 端到端：缺 id payload 导入后结构校验

- **GIVEN**：mock 后端；payload 为 `{tables:[{name,x,y,fields:[{name,type}]}]}`（表/字段均缺 id，无 name）
- **WHEN**：stdio → `import_schema` → `get_diagram(diagram_id)`
- **THEN**：import 响应 `imported_tables == 1`；get 响应 `diagram.tables[0].id` 非空（auto- 前缀）、`diagram.tables[0].fields[0].id` 非空、`diagram.name` 为 `"imported_diagram"`（或 payload.name）；`structuredContent` 通过各自 outputSchema 校验
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S06"`）
