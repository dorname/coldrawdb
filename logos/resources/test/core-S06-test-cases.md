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
| UT-MCP-06 | 完整 fixture：`users` 为单主键；`membership(tenant_id,user_id)` 为复合主键；字段含 NOT NULL、UNIQUE、DEFAULT；表/字段注释含单引号；`membership.user_id → users.id` 含 `on_delete=CASCADE`、`on_update=SET_NULL`；另含一条悬空关系。导出 JSON/DBML/七类 SQL | PostgreSQL 单主键保持列内、复合主键仅一个表级声明，NOT NULL/UNIQUE/DEFAULT/FK/ON DELETE/ON UPDATE/COMMENT ON 齐全且注释正确转义；DBML 含复合主键 indexes、字段 settings、表/字段 note、有效 Ref 及级联；MySQL/MariaDB 使用方言 COMMENT，SQLite/MSSQL 无 COMMENT ON；七类 SQL 的复合表均无列级 PRIMARY KEY；悬空关系跳过且无 `?`；相同输入输出字节一致，JSON/envelope 不变，无数据库或网络副作用 |
| UT-MCP-07 | create 请求 | 只发送 name/database；解析 `{code,data,request_id}` |
| UT-MCP-08 | update 请求 | 显式发送 expected_revision 和完整 diagram；成功 revision +1 |
| UT-MCP-09 | delete confirm=false/缺失 | 本地拒绝；不发送 HTTP；confirm=true 才允许 |
| UT-MCP-10 | import payload | 非 object 本地拒绝；drawdb_json object 正确映射 |
| UT-MCP-11 | 四客户端配置 fixture | Claude/Cursor JSON、Codex TOML、OpenCode JSON 可解析且命令/环境一致 |
| UT-MCP-12 | 上游 409 | 映射 `REVISION_CONFLICT`，保留 current_revision，retryable=false |
| UT-MCP-13 | 400/401/403/404/422/5xx | 映射表一致；只暴露白名单 details/request_id |
| UT-MCP-14 | 连接失败/超时 | 分别映射 `UPSTREAM_UNREACHABLE` / `UPSTREAM_TIMEOUT`；写操作不自动重试 |
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
| ST-MCP-02 | mock 上游返回 UT-MCP-06 的完整 diagram；list → get → 分别 export JSON/DBML/PostgreSQL 及其余六类 SQL → 对相同输入重复导出 | 所有响应 id/revision 一致；get 保留完整模型；PostgreSQL/DBML 逐项满足复合主键、字段约束、默认值、注释、FK、级联和悬空关系跳过断言；其余 SQL 无重复列级主键且方言注释正确；重复导出内容字节一致，envelope/MIME 不变 |
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
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、ST-MCP-03/14/18 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、UT-MCP-45～49、ST-MCP-04/05/19～21 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、UT-MCP-47、ST-MCP-06～09/20 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27、UT-MCP-44～48、ST-MCP-19/20 |

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

- **位置**：`mcp-server` 协作 diff / 会话逻辑。
- **GIVEN**：活跃房间 `diagramId` 匹配；基准表名与目标表名不同；假 socket 先回 `connected` 再回 `ack.serverRev`。
- **WHEN**：规划并提交房间写入。
- **THEN**：产生不含 fields 的 `table.update`；提交后 revision 为 ack 的 serverRev；`READ_ONLY` 在第一帧失败且不再继续发送；空 diff 保持当前 revision。
- **#71 修订**：无凭据的房间写回退不再断言笼统 `UNAUTHENTICATED`，而是在 GET rooms 前返回 `ROOMS_AUTH_REQUIRED`，details.reason 为 `TOKEN_MISSING`，详见 UT-MCP-45；普通非房间 API 401 仍保持 `UNAUTHENTICATED`。

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

---

## 合并自 fix-open-issues-26-33（2026-09-27）

## ADDED — UT 用例（追加）

> 改号注记：本批 UT-MCP-23/24/25、ST-MCP-13/14 与既有同名用例（写工具瘦响应/color/USE_OP_CHANNEL、layout force/import 缺 id）撞号，按场景编号全局唯一规则顺移至 UT-MCP-30/31/32、ST-MCP-15/16。

| UT-MCP-30 | `update_diagram` 参数体积估算与软上限拦截 | 体积恰 ≤ 软上限 → 直写不拦截；体积 > 软上限 → 返回 `PAYLOAD_TOO_LARGE`，`isError=true`，details 含 `payload_bytes` / `soft_limit_bytes` / `suggestion` / `batch_hint`；**mock HTTP 断言零请求发出**；diagram 数据不变 |
| UT-MCP-31 | 软上限配置生效 | `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES` 覆盖默认值；非法值（< 16 KiB / 非整数）→ 按默认 256 KiB 处理并 warn（stderr 脱敏行）；retryable=false |
| UT-MCP-32 | `batch_hint` 构成建议 | 变更以表为主 → `suggested_batch=update_table`；以关系为主 → `update_reference`；纯布局 → `layout_diagram`；details 中 `tables` / `references` 计数与 diagram 实际一致 |

## ADDED — ST 用例（追加）

| ST-MCP-15 | 启动 stdio → 构造超软上限的大图变更 → `update_diagram` 被拦截 → 按指引分批（update_table × N → update_reference × M → layout_diagram），每批携带最新 `expected_revision` 串行 → get_diagram 验证 | 拦截响应结构正确；分批全部成功；revision 连续递增无跳号；最终 diagram 与目标变更一致 |
| ST-MCP-16 | 分批链路中人为制造一批 409（过期 expected_revision）→ 重试该批 | 失败批返回 `REVISION_CONFLICT`；已成功批次不回滚；以最新 revision 重试后该批成功；最终状态一致、无数据损坏 |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-30 | UT | update_diagram 软上限拦截 + PAYLOAD_TOO_LARGE 结构 + 零请求 |
| UT-MCP-31 | UT | 软上限环境变量配置生效与非法值回退 |
| UT-MCP-32 | UT | batch_hint 构成建议与计数 |
| ST-MCP-15 | ST | 超限拦截后按官方分批指引串行写成功 |
| ST-MCP-16 | ST | 分批链路单批 409 重试不损坏 revision |

---

## 合并自 mcp-dictionary-tools（2026-09-28）

## MODIFIED — UT 用例（UT-MCP-20 行改写）

| UT-MCP-20 | tools/list 工具总数 | 恰好 12 个工具；新增 5 个工具（update_table/update_field/update_reference/layout_diagram/update_dictionary）name/title/description/inputSchema/outputSchema 稳定 |

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-33 | `update_dictionary` create + 编码唯一校验 | create 缺 name/code → `VALIDATION_ERROR` 零请求；code 与既有字典冲突 → `VALIDATION_ERROR` 零请求（对齐 S07 EX-7.1）；合法 create（含 items，缺 id 项自动补 id）→ PUT body `dictionaries` 追加一条且其余图元不变，瘦响应含 `{id, revision, dict_id, fields_cleared:0}` |
| UT-MCP-34 | `update_dictionary` update 改码同步引用 + delete 级联置空 | update 改 code → 所有 `dict_code` 指向旧编码的字段同步改写为新编码（无悬空）；改 code 与另一字典冲突 → `VALIDATION_ERROR`；delete → 字典移除且引用字段 `dict_code` 置空，瘦响应 `fields_cleared` = 实际置空字段数（对齐 S07 §5）；dict_id 不存在 → `VALIDATION_ERROR` 零请求 |
| UT-MCP-35 | `update_field` dict_code 绑定/解绑/悬空拒绝 | 绑定已存在字典编码 → PUT body 该字段 `dict_code` 更新；`dict_code=""` → 解绑置空；绑定不存在编码 → `VALIDATION_ERROR` 且 mock HTTP 断言零请求发出（杜绝 S07 EX-7.4 悬空绑定） |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-33 | UT | update_dictionary create + 编码唯一校验 + 瘦响应结构 |
| UT-MCP-34 | UT | update_dictionary update 改码同步引用 + delete 级联置空 fields_cleared |
| UT-MCP-35 | UT | update_field dict_code 绑定/解绑/悬空拒绝零请求 |

## 合并自 fix-issues-42-44-mcp-and-relation-hit（2026-09-28）

## ADDED — UT 用例（追加，#42）

| ID | 场景 | 预期 |
|---|---|---|
| UT-MCP-36 | mock 上游 409 返回**非 JSON** 文本但含 `USE_OP_CHANNEL` 字样 → `update_diagram` | 识别为房间图收编，走 WS op 通道回退（不落入笼统 VALIDATION_ERROR）（R-MCPERR-02） |
| UT-MCP-37 | mock 上游 502 返回 HTML 错误页 → `update_diagram` | 错误 details 含 `http_status=502`、真实 `content_type`、`body_excerpt`（≤200 字符摘要）；message 可辨认网关错误（R-MCPERR-01） |
| UT-MCP-38 | 读取 `import_schema` tool 描述 | 明确含「仅新建/不覆盖已有 id」语义文案（R-MCPERR-04） |

- **reporter**：上述 UT 均写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-36 | UT | 非 JSON 409 含 USE_OP_CHANNEL 字样仍走 op 通道（#42） |
| UT-MCP-37 | UT | 非 JSON 上游错误 details 携带 status/content_type/body_excerpt（#42） |
| UT-MCP-38 | UT | import_schema tool 描述「仅新建」文案（#42） |

## 合并自 fix-reference-revival-and-mcp-errors（2026-10-30）

### UT-MCP-39 — HTTP 连接失败包含 BASE_URL 诊断

- **位置**：`mcp-server/tests/c10_error_diagnosis.rs`。
- **操作**：`COLDRAWDB_BASE_URL` 指向未监听的本地端口，调用 `get_diagram`。
- **断言**：返回 `UPSTREAM_UNREACHABLE`、`retryable=true`；message 与 `details.base_url` 均包含配置的 origin；不包含 token、userinfo、query 或 fragment；不映射为成功 outputSchema 的字段错误。

### ST-MCP-17 — stdio 工具错误不再触发成功 outputSchema 校验

- **位置**：`mcp-server/tests/c10_error_diagnosis.rs`，真实 `coldrawdb-mcp` stdio 子进程。
- **操作**：以未监听端口为 BASE_URL，完成 initialize 后调用 `get_diagram`。
- **断言**：JSON-RPC 结果为 `result.isError=true`；不存在 `result.structuredContent`；`content[0].text` 可解析为完整 ToolError，含 `UPSTREAM_UNREACHABLE` 与 BASE_URL；不会出现成功 schema 所需的 `id` 伪字段，stdout 无其他非协议内容。
- **reporter**：分别登记 `UT-MCP-39`、`ST-MCP-17` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-39 | UT | HTTP 连接失败返回 UPSTREAM_UNREACHABLE 和脱敏 BASE_URL |
| ST-MCP-17 | ST | stdio 错误结果仅文本 content，无 structuredContent |

既有错误测试中读取 `result.structuredContent` 的断言同步改为解析 `result.content[0].text`；成功响应断言保持不变。

## 合并自 fix-diagram-list-metadata（2026-10-31）

### UT-MCP-40 — list_diagrams 保留真实摘要元数据

- **位置**：`mcp-server/tests/c2_read_tools.rs`。
- **输入**：上游列表项包含 `revision:8`、`updated_at:'2026-10-29T12:34:56Z'` 和不同的 `lastModified`；另含仅提供旧 `lastModified` 的兼容项。
- **操作**：执行 `normalize_list`。
- **断言**：新字段项的 revision/updated_at 原样保留，不退化为 0/null，updated_at 优先于 lastModified；旧字段项仍回退到 lastModified；名称过滤、排序、limit 行为不变。
- **reporter**：登记 `UT-MCP-40` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-40 | UT | list_diagrams 透传真实 revision/updated_at，并保留 lastModified 回退 |

## 合并自 fix-issues-70-73（2026-11-02）

## ADDED — fix-issues-70-73：MCP 分片、凭据与端点用例

### UT-MCP-41 — update_table create 整表追加与冲突安全 ID 映射

- **位置**：`mcp-server/tests/c11_chunked_import.rs`。
- **操作**：以 `action=create` 写入一张含完整 fields 的表；字段覆盖 comment、increment、unique、check、tag、dict_code、default、primary、not_null；另测重复 ID、缺 name/fields、update 携带 fields。
- **断言**：合法 create 仅一组 GET/PUT，旧表/关系不变，字段全保真；实际 table/field ID 采用冲突安全重键，响应在 `{id,revision}` 外包含 `table_id` 与 `field_id_map`；非法输入在写前拒绝且零 PUT；缺省 action 仍为 update。

### UT-MCP-42 — update_reference 批量 create 单次快照写

- **操作**：`action=create` 携带多条 `references`，包含显式 ref_id 与缺省 ref_id；另测批内重复、既有 ID 冲突、非法枚举，以及平铺单条字段与 references 数组同时出现。
- **断言**：合法批次一次 GET、一次 PUT 追加全部关系，非房间 revision +1；显式 ID 保留，缺省 ID 唯一；两种 create 形态互斥，任一非法项导致整批零 PUT；旧平铺 create/update/delete 回归不变。

### UT-MCP-43 — 关系 type_ 与级联值规范化

- **操作**：分别执行平铺 create、update 和批量 create，传入 cardinality 及小写 on_delete/on_update。
- **断言**：PUT 快照消费 `type_`，不再以会被 Serde 丢弃的 `cardinality` 代替；级联值显式映射为 `CASCADE` / `SET_NULL` / `SET_DEFAULT` / `RESTRICT` / `NO_ACTION`；缺省值经过同一映射；mock GET 中的原字段不被误当作写入证据。

### UT-MCP-44 — 房间整表与批量关系 diff

- **操作**：对房间图执行整表 create 与批量关系 create，使用假 socket 验证 op 计划和逐条 ack。
- **断言**：一张新表产生一个含完整 fields 的 `table.create`，不产生重复 `field.create`；N 条新关系产生 N 个 `reference.create`；最终 revision 来自最后 ack；不得宣称多 op 是单事务或 revision 只加 1。

### UT-MCP-45 — 房间写缺少凭据的上下文化错误

- **操作**：PUT 返回 `USE_OP_CHANNEL`，且静态 token 与动态登录均未配置。
- **断言**：在 GET rooms 前返回 `ROOMS_AUTH_REQUIRED`，details.reason=`TOKEN_MISSING`，消息明确房间图 MCP 写前置条件；rooms 请求数为零；个人图匿名写与普通 401 语义不变。

### UT-MCP-46 — rooms 401 保留安全根因

- **操作**：房间回退的 GET rooms 返回 401，响应含 request_id；注入敏感 token/cookie。
- **断言**：返回 `ROOMS_AUTH_REQUIRED`，details.reason=`TOKEN_INVALID_OR_EXPIRED`，保留 `upstream_code=UNAUTHENTICATED`、http_status=401 与 request_id；token、password、Cookie 不出现在 message/details/stderr/reporter；403 仍为 PERMISSION_DENIED，5xx 不重写为该错误。

### UT-MCP-47 — login/refresh 成功后单次重试

- **操作**：以 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD` 建立内存会话；首个 access token 返回 401，refresh 返回新 token 与轮换 cookie；随后重试 HTTP 写并建立 WS。
- **断言**：login/refresh 使用现有 S03 契约；原请求最多重试一次且 refresh 不递归；后续 HTTP Authorization 与 WS query 使用最新 token；静态 token 与动态登录同时配置时本地拒绝；敏感值零泄漏。

### UT-MCP-48 — refresh 失败与 WS 握手边界

- **操作**：覆盖 refresh 401/网络失败、WS 握手 401，以及已经发送 op 后的失败。
- **断言**：refresh 失败不产生重试风暴、不执行写 op，返回可诊断且脱敏的根因；WS 仅在尚未发送 op 的握手 401 时 refresh 并重连一次；已发送 op 后不盲目重放。

### UT-MCP-49 — 关系端点存在性与字段归属校验

- **操作**：mock GET 返回重键后的 auto-* 表/字段；分别用导入前旧 ID、不存在的 ID、存在但不属于指定表的字段执行单个与批量 create。
- **断言**：均返回 `VALIDATION_ERROR`，details 指出无效端点，PUT/WS 均为零；使用 GET 返回的实际 ID 且字段归属正确时成功；批量任一项非法则整批不写。

### ST-MCP-18 — 真实 backend + stdio 的 54 表全保真分片

- **位置**：`mcp-server/tests/c11_chunked_import.rs`，启动真实隔离 backend 与 `coldrawdb-mcp` stdio 子进程，不得只用 mock 或声明式 reporter。
- **夹具**：运行时确定性生成 54 表 / 863 字段 / 76 关系，原始全保真 JSON 超过 63,284 B；每次工具参数控制在约 48 KiB 以下。
- **步骤**：create_diagram → 每表 update_table(create) 并收集实际 ID 映射 → 按预算分批 update_reference(create,references) → get_diagram。
- **断言**：最终数量精确为 54/863/76；字段注释、自增、唯一、check、tag、表颜色与坐标无损；关系基数及 CASCADE/SET_NULL 真实往返正确；所有 stdio 响应通过 outputSchema；reporter 记录非零实际耗时和真实进程断言。

### ST-MCP-19 — 真实 stdio 房间写缺 token 错误

- **操作**：真实房间图触发 PUT 409 USE_OP_CHANNEL，未配置任何凭据。
- **断言**：`result.isError=true`；仅 `content[0].text` 携带完整 ToolError，无 `structuredContent`；包含 `ROOMS_AUTH_REQUIRED/TOKEN_MISSING`，无敏感信息；未发送 GET rooms。

### ST-MCP-20 — login→过期→refresh→房间 op ack

- **操作**：真实隔离 backend 使用可控短 access TTL；MCP 动态登录后执行房间写，使首个 token 过期并触发 refresh。
- **断言**：login→401→refresh 单次→原请求重试→PUT 409→GET rooms→WS ack 全链路成功；refresh cookie 轮换后后续调用仍可用；最终物化文档与目标一致；输出不含密码、token 或 cookie。

### ST-MCP-21 — 导入到房间的存量关系端点保真

- **操作**：import_schema 导入含表/字段但无关系的图 → get_diagram 获取实际 auto-* ID → update_reference 建边 → 创建房间并提交首 op → 再 GET；另用导入前旧 ID 建边。
- **断言**：实际 ID 建边在物化前后数量不减少，四端点全部可解析，table/field/reference/area/note 保持；旧 ID 建边被明确拒绝而非成功落库后丢线。

## ADDED — 用例登记（OpenLogos verify 解析用，fix-issues-70-73）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-41 | UT | 整表 create、全字段保留、冲突安全 ID 映射与非法输入零写入 |
| UT-MCP-42 | UT | 批量关系 create 单次 GET/PUT、形态互斥与整批校验 |
| UT-MCP-43 | UT | 单个/批量关系 type_ 与大写级联规范化 |
| UT-MCP-44 | UT | 房间整表/批量关系 diff 与逐 op ack |
| UT-MCP-45 | UT | 房间写缺凭据返回 ROOMS_AUTH_REQUIRED/TOKEN_MISSING |
| UT-MCP-46 | UT | rooms 401 根因上下文化与敏感信息脱敏 |
| UT-MCP-47 | UT | 动态 login/refresh、单次重试与最新 HTTP/WS token |
| UT-MCP-48 | UT | refresh 失败无重试风暴、WS 仅握手前重连 |
| UT-MCP-49 | UT | 关系端点存在性/字段归属与无效输入零写入 |
| ST-MCP-18 | ST | 真实 backend+stdio 的 54 表/863 字段/76 关系全保真分片 |
| ST-MCP-19 | ST | 真实 stdio 房间写缺 token 仅文本错误 |
| ST-MCP-20 | ST | 真实 login/refresh/房间 op ack 全链路 |
| ST-MCP-21 | ST | 导入→实际 ID→建边→房间物化端点保真 |

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、ST-MCP-03/14/18 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、UT-MCP-45～49、ST-MCP-04/05/19～21 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、UT-MCP-47、ST-MCP-06～09/20 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27、UT-MCP-44～48、ST-MCP-19/20 |

## 合并自 fix-mcp-tool-schema-client-compat（2026-11-03）

## ADDED — UT-MCP-50 — 工具 schema 客户端兼容守卫

- **位置**：`mcp-server/tests/c6_write_contract.rs`，纯本地断言 `McpService::tools()`，不依赖后端。
- **GIVEN**：`McpService::tools()` 返回的全部工具（应为 12 个）。
- **WHEN / THEN**：
  1. 遍历每个工具的 `inputSchema` 与 `outputSchema`，顶层键不得包含 `allOf`、`oneOf`、`anyOf`、`if`、`then`、`else`、`not`（客户端兼容约束；含任一即该客户端会丢弃工具）。
  2. `update_table.inputSchema.properties` 必须仍包含 `id`、`table_id`、`action`、`name`、`fields`（拍平不丢字段），`fields.items` 的 `$ref` 已解析为内联 schema（含 `name`/`type` 属性）。
  3. `update_reference.inputSchema.properties` 必须仍包含 `id`、`action`、`ref_id`、`references`、四个端点字段与 `cardinality`/`on_delete`/`on_update`；`references.items` 解析后含 `start_table_id` 等端点属性。
  4. 工具总数为 12，且包含 `update_table` 与 `update_reference`（回归锚点：客户端清单不得再退化为 10）。
- **reporter**：测试真实运行后登记 `UT-MCP-50` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — 用例登记（OpenLogos verify 解析用，fix-mcp-tool-schema-client-compat）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-50 | UT | 全工具 inputSchema/outputSchema 顶层客户端兼容守卫与拍平后关键属性保留 |

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09、UT-MCP-50 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、UT-MCP-50、ST-MCP-03/14/18 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、UT-MCP-45～49、ST-MCP-04/05/19～21 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、UT-MCP-47、ST-MCP-06～09/20 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27、UT-MCP-44～48、ST-MCP-19/20 |

## 合并自 fix-mcp-tool-gaps-issue76（2026-11-04）

## ADDED — UT-MCP-51 — update_table action=delete 删表与引用清理

- **位置**：`mcp-server/tests/c6_write_contract.rs`，mock 后端（GET + PUT 序列）。
- **GIVEN**：图中 t1（含字段 f1/f2）、t2，r1 连接 t1→t2，r2 连接 t2→t1。
- **WHEN / THEN**：
  1. `update_table(action=delete, table_id=t1)` 成功：PUT 载荷 tables 仅剩 t2；references 为空（r1/r2 均触碰 t1 被清理）；响应含 `deleted_table_id="t1"` 与 `deleted_reference_ids=["r1","r2"]`。
  2. `action=delete` 且携带 `name` → VALIDATION_ERROR，零写入（无 PUT）。
  3. `action=delete` 且 `table_id` 不存在 → VALIDATION_ERROR，零写入。
- **reporter**：真实运行后登记 `UT-MCP-51`。

## ADDED — UT-MCP-52 — update_area 三动作（create/update/delete）

- **位置**：同上。
- **WHEN / THEN**：
  1. create：必填 name，缺省 x/y/width/height/color/locked 落默认值；服务端重键 auto-，响应 `area_id` 为实际 ID；PUT 载荷 areas 含新区域全字段。
  2. update：按实际 area_id 改 name+width，未携带属性（x/y/color/locked）保持原值；area_id 不存在 → VALIDATION_ERROR 零写入。
  3. delete：区域从 PUT 载荷消失；携带禁字段（如 name）→ VALIDATION_ERROR 零写入。
- **reporter**：真实运行后登记 `UT-MCP-52`。

## ADDED — UT-MCP-53 — update_note 三动作（create/update/delete）

- **位置**：同上。
- **WHEN / THEN**：
  1. create：必填 content，服务端重键，响应 `note_id` 为实际 ID；PUT 载荷 notes 含新便签。
  2. update：按实际 note_id 改 content+color，x/y 保持原值；note_id 不存在 → VALIDATION_ERROR 零写入。
  3. delete：便签从 PUT 载荷消失；携带禁字段 → VALIDATION_ERROR 零写入。
- **reporter**：真实运行后登记 `UT-MCP-53`。

## ADDED — UT-MCP-54 — update_diagram 防误删守护（allow_mass_delete）

- **位置**：同上（GET base + 校验本地拦截，不发上游写）。
- **WHEN / THEN**（base 3 表 t1/t2/t3）：
  1. desired 缺 t1（缺失 1/3 ≤50% 且 ≤10）→ 正常放行。
  2. desired 仅含 t1（缺失 2/3 >50%）且无 allow_mass_delete → VALIDATION_ERROR，details 含缺失计数，零写入（无 PUT、无 WS）。
  3. 同 2 但 `allow_mass_delete=true` → 放行执行。
- **reporter**：真实运行后登记 `UT-MCP-54`。

## MODIFIED — UT-MCP-50 — 工具 schema 客户端兼容守卫（锚点更新）

断言 4 的工具总数锚点 12 → **14**，且包含 `update_area` / `update_note`；
断言 1 扩展：新工具 update_area/update_note 的 schema 同样整树无
allOf/oneOf/anyOf/if/then/else/not。

## ADDED — ST-MCP-22 — 真实 stdio tools/list 14 工具探针

- **位置**：`mcp-server/tests/`（既有 stdio 探针模式，如 c6 ST-MCP-14 同款子进程 + JSON-RPC）。
- **WHEN / THEN**：release 二进制真实 stdio `tools/list` 返回 14 个工具，含
  `update_table`（action enum 含 delete）、`update_area`、`update_note`；
  新工具 schema 抽样断言平铺结构（顶层 type/properties/required）。
- **reporter**：真实运行后登记 `ST-MCP-22`。

## ADDED — 用例登记（OpenLogos verify 解析用，fix-mcp-tool-gaps-issue76）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-51 | UT | update_table delete 删表、触碰 references 清理、禁字段与不存在零写入 |
| UT-MCP-52 | UT | update_area create/update/delete 全动作与校验 |
| UT-MCP-53 | UT | update_note create/update/delete 全动作与校验 |
| UT-MCP-54 | UT | update_diagram 缺表守护三态（放行/拦截/确认放行） |
| ST-MCP-22 | ST | 真实 stdio tools/list 14 工具与新工具 schema 平铺探针 |

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09、UT-MCP-50 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、UT-MCP-50、UT-MCP-51～53、ST-MCP-03/14/18 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、UT-MCP-45～49、UT-MCP-54、ST-MCP-04/05/19～21 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、UT-MCP-47、ST-MCP-06～09/20、ST-MCP-22 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27、UT-MCP-44～48、ST-MCP-19/20 |
