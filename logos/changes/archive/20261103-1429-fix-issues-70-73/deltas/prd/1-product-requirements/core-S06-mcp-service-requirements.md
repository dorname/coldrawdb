# S06：AI 客户端通过 MCP 管理数据库图表 — 产品需求 Delta

> 提案：fix-issues-70-73 | 关联 issue：#70、#71、#72
> 合并规则：以下 `MODIFIED` 章节完整替换主文档同号章节；未列章节保持不变。

## MODIFIED — §2 What：功能需求

| ID | 需求 | 验收 |
|---|---|---|
| FR-S06-01 | 提供 MCP stdio 服务并完成 initialize、initialized、tools/list | 四客户端配置可启动服务；tools/list 返回十二个稳定工具 |
| FR-S06-02 | 提供 `list_diagrams`、`get_diagram` | 返回结构化摘要或完整 diagram；列表摘要必须返回上游真实 `revision` 与 `updated_at`，不得因遗留 VO 缺列退化为 `0/null`；不存在时返回可诊断错误 |
| FR-S06-03 | 提供 `create_diagram`、`update_diagram`、`delete_diagram`，以及画布写工具；支持整表与关系数组的无损分片，并拒绝无效关系端点 | 非房间图 update 走 `PUT /api/v1/diagrams/{id}` 且必须携带 `expected_revision`；房间绑定图在 PUT 返回 `USE_OP_CHANNEL` 后改经 `/ws/rooms/{roomId}` 提交 op。`update_table(action=create)` 一次追加一张含完整 fields 的表，为表和字段生成冲突安全的服务端实际 ID，并返回实际 `table_id` 与 field 旧→新映射；缺省 `action=update` 保持旧行为且不接受 fields。`update_reference(action=create)` 支持互斥的平铺单条或 `references` 数组批量形态；外部 `cardinality` 必须写入后端 `type_`，级联动作统一为大写下划线。单个/批量 create 必须使用服务端实际表/字段 ID，并在写前验证四个端点存在及字段归属；无效端点在 PUT/WS 前返回 `VALIDATION_ERROR` |
| FR-S06-04 | 提供 `import_schema` | MVP 接受 drawdb JSON；SQL/DBML 文本导入仅在解析能力落地后开放对应枚举值；导入后继续建关系时必须重新读取并使用服务端实际 ID，不得复用导入前旧 ID |
| FR-S06-05 | 提供 `export_schema` | 从完整 diagram 在 adapter 内确定性生成 JSON、DBML 或七类 SQL；SQL/DBML 保留单/复合主键、NOT NULL、UNIQUE、DEFAULT、有效 FK 及级联动作和注释，复合主键不得生成多个列级 `PRIMARY KEY` |
| FR-S06-06 | 提供四客户端配置 | Claude、Codex、Cursor、OpenCode 示例可复制，均指向同一二进制；静态 token 与动态登录配置示例不写入真实凭据 |
| FR-S06-07 | 标注工具副作用 | 读工具 `readOnlyHint=true`；删除工具 `destructiveHint=true`；写工具不伪装为只读 |
| FR-S06-08 | 透传上游错误并提供可持续的房间凭据会话 | 网络、400/401/403/404/409/422/5xx 映射为稳定 MCP 错误码并保留安全的 request_id/details；房间写收编 409 的 `details.code=USE_OP_CHANNEL` 不得映射为 `REVISION_CONFLICT`。统一房间回退点对无凭据或 rooms 401 返回 `ROOMS_AUTH_REQUIRED`，details.reason 区分 `TOKEN_MISSING` / `TOKEN_INVALID_OR_EXPIRED`；403 与 5xx 不混淆。静态 `COLDRAWDB_ACCESS_TOKEN` 与 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD` 二选一；动态模式复用 login/refresh，在内存维护 access token 与 refresh cookie。动态会话 API 401 最多 refresh 一次并重试原请求一次；WS 仅握手 401 且尚未发送 op 时 refresh 并重连一次，不重放已发送 op |
| FR-S06-09 | 提供大图分批写入官方约定与超限可诊断错误 | `update_diagram` 保留既有载荷软上限和发送前 `PAYLOAD_TOO_LARGE` 拦截；工具描述与文档建议整表一次一片、关系按数组分片，单次工具参数保守控制在约 48 KiB，并严格串行携带最新 `expected_revision`。约 63 KB 仅为特定客户端实测边界，不作为通用硬限制；48 KiB 为建议值，不替代既有软上限。跨调用不回滚、不宣称整链原子；房间批量关系逐 op ack，不承诺整批事务或 revision 只增加 1 |

## MODIFIED — §3 非功能需求

| ID | 约束 | 指标 |
|---|---|---|
| NFR-S06-01 | 启动性能 | 本地 release 二进制 initialize P95 ≤ 1s；动态 login 可在首次需要鉴权的调用时执行，不阻塞无鉴权读工具初始化 |
| NFR-S06-02 | 调用超时 | 默认 30s，可通过 `COLDRAWDB_REQUEST_TIMEOUT_SECS` 配置为 1～120s；login/refresh 使用同一上游超时边界 |
| NFR-S06-03 | 安全与日志 | stdout 只写 MCP 帧；成功 tool_call 不写 stderr；失败时 stderr 仅一行 JSON，含 `code` 与 `message`；密码、Authorization、access token、refresh cookie、WebSocket query token 必须脱敏。密码与 refresh cookie 不得进入 URL；access token 仅按既有 Authorization header 与 WS 握手 query 契约传输，所有日志、reporter、错误摘要和诊断 URL 必须移除秘密 |
| NFR-S06-04 | 数据边界 | 禁止 SQLite 文件访问、任意 SQL 执行和 shell 执行工具；动态凭据仅访问既有固定 login/refresh 端点 |
| NFR-S06-05 | 兼容 | MCP 客户端协商版本，不硬编码拒绝未知的新版本；工具 schema 保持向后兼容：`update_table` 缺省 action 仍为 update，旧平铺 `update_reference` 不变，静态 token 路径保留，工具总数仍为 12 |
| NFR-S06-06 | 可测试 | 每个工具至少一个 UT；主链、409、配置兼容、整表/批量分片、refresh/WS 重连和无效端点零写入具有 ST；全部写 OpenLogos reporter，禁止声明式假入账 |

## MODIFIED — §4.1 MVP 范围

- 仅 stdio transport；进程由客户端启动和停止。
- 仅连接 `COLDRAWDB_BASE_URL` 指定的 HTTP 服务；动态凭据只复用既有 `POST /api/v1/auth/login` 与 `POST /api/v1/auth/refresh`，不新增 REST 端点，不改变 S03 Cookie 契约。
- `COLDRAWDB_ACCESS_TOKEN` 可选，配置后以 Bearer header 透传；它与 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD` 动态登录模式互斥。动态模式的 email 与 password 必须成对配置，access token 和可由服务端轮换的 refresh cookie 仅保存在进程内存，不写配置文件或持久化存储。
- 未配置任何凭据仍可使用无需鉴权的既有 diagram API；进入房间写回退时返回 `ROOMS_AUTH_REQUIRED` / `TOKEN_MISSING`，不先发起必失败的 rooms 请求。
- `list_diagrams` 暂映射遗留只读端点 `GET /diagrams/queryAll`；其余 CRUD/导入映射 `/api/v1/diagrams*`。
- `export_schema` 先读取完整 diagram，再在 adapter 内进行纯函数序列化，不访问数据库。
- 大图分片只扩展现有 12 个工具的输入与必要输出，不新增 batch 工具或 partial update REST API；整表 create 返回服务端实际 ID，后续关系引用这些 ID。
- 不要求用户复制 HttpOnly refresh token，不新增 service token 表、长期高权 token、全局 JWT TTL 延长或数据迁移。

## MODIFIED — §5 验收场景

- GIVEN coldrawdb API 正常且客户端已配置 stdio 服务，WHEN 客户端完成 MCP 初始化，THEN 能发现十二个工具及正确 annotations。
- GIVEN 已存在包含单/复合主键、字段约束、默认值、注释和关系的 diagram，WHEN 调用 `get_diagram` 与 `export_schema(format="dbml")`，THEN 返回同一 revision 对应的完整模型和确定性 DBML；复合主键使用表级 indexes，字段约束、note 与有效 Ref（含级联动作）均被保留。
- GIVEN 同一完整 diagram，WHEN 调用 `export_schema(format="postgresql")`，THEN 单列主键保持列内声明，复合主键只生成一个表级 `PRIMARY KEY (...)`；NOT NULL、UNIQUE、DEFAULT、FOREIGN KEY、ON DELETE、ON UPDATE 与 COMMENT ON 均符合源模型。
- GIVEN 同一 diagram 分别导出 MySQL、MariaDB、SQLite、MSSQL、Oracle 和 generic SQL，WHEN 检查约束与注释，THEN 各格式均无重复列级主键且保留有效约束和关系；MySQL/MariaDB 使用方言 COMMENT，SQLite/MSSQL 不输出不兼容的 COMMENT ON。
- GIVEN diagram 含无法解析表或字段端点的悬空关系，WHEN 导出 SQL 或 DBML，THEN 跳过该关系，不生成 `?` 伪引用，且相同输入的重复导出字节一致。
- GIVEN `expected_revision` 已过期，WHEN 调用 `update_diagram`，THEN 返回 `REVISION_CONFLICT` 和 `current_revision`，不自动覆盖。
- GIVEN 一次 `update_diagram` 的序列化参数体积超过软上限，WHEN 客户端调用该工具，THEN 返回 `PAYLOAD_TOO_LARGE` 且 details 给出当前体积、上限与建议分片粒度，不发送上游请求、diagram 数据不变。
- GIVEN `update_table(action=create)` 接收一张含完整 fields 的表，WHEN 服务端保存成功，THEN 只追加一张表，表和字段均使用冲突安全的服务端实际 `auto-` ID，响应含实际 `table_id` 和完整 field 旧→新映射；原调用方 ID 不被假定已落库。
- GIVEN `update_table` 未传 action，WHEN 使用既有表属性参数调用，THEN 按 update 处理；WHEN update 携带 fields，THEN 在写前返回 `VALIDATION_ERROR`。
- GIVEN 一张全保真 54 表 / 863 字段 / 76 关系的大图，WHEN 客户端按单表一片、关系数组分片且单次参数约不超过 48 KiB 的建议严格串行写入，THEN 最终表、字段、关系及端点完整，无裁剪、无旧 ID 悬空引用；约 63 KB 客户端实测值不被写成所有客户端的硬限制。
- GIVEN `update_reference(action=create)` 同时携带 `references` 与任一平铺关系字段，或在 update/delete 中携带 `references`，或 `references` 为空，WHEN 调用工具，THEN 返回 `VALIDATION_ERROR` 且不发送写请求。
- GIVEN 单个或批量关系使用小写外部 `cardinality` 与级联枚举，WHEN 构造写回快照，THEN 基数写入 `type_` 而非 `cardinality`，`on_delete` / `on_update` 分别为 `CASCADE`、`SET_NULL`、`SET_DEFAULT`、`RESTRICT`、`NO_ACTION` 之一；单条与批量结果一致。
- GIVEN 批量关系中任一项 schema、枚举或端点校验失败，WHEN 调用 `update_reference(action=create, references=[...])`，THEN 整批不发送 PUT/WS、revision 不变，已成功的前序调用不回滚。
- GIVEN 非房间批量关系全部合法，WHEN 写回成功，THEN 该批通过一次快照 PUT 追加；GIVEN 房间批量关系全部合法，THEN 经多个 op 逐条等待 ack，不宣称跨 op 事务、跨调用回滚、整链原子或 revision 只增加 1。
- GIVEN `import_schema` 或整表 create 已重键，WHEN 后续创建关系，THEN 调用方通过 `get_diagram` 或 create 响应取得实际 table/field ID；旧 ID、悬空 ID、字段不属于指定表均在 PUT/WS 前返回 `VALIDATION_ERROR`，且单个/批量 create 均为零写入。
- GIVEN 使用实际 ID 创建关系后进入房间并提交首个 op，WHEN 再次读取物化图，THEN table/field/reference/area/note ID 与关系端点保持服务端实际值，物化阶段不二次重键、不猜测旧映射。
- GIVEN diagram 绑定活跃协作房间且 `expected_revision` 等于当前物化 revision，WHEN 调用写工具，THEN adapter 经 WebSocket 提交 op 并返回新的 `revision`；viewer 得到 `READ_ONLY`。
- GIVEN 房间写进入统一 `USE_OP_CHANNEL` 回退但未配置凭据，WHEN adapter 识别 `TOKEN_MISSING`，THEN 返回 `ROOMS_AUTH_REQUIRED` 且不先调用必失败的 `GET /api/v1/rooms`；WHEN rooms 返回 401，THEN 返回 `ROOMS_AUTH_REQUIRED` 且 `details.reason=TOKEN_INVALID_OR_EXPIRED` 并保留安全上游诊断；普通非房间 401 仍为 `UNAUTHENTICATED`，403 与 5xx 保持原语义。
- GIVEN 同时配置静态 token 与任一动态登录变量，或只配置 email/password 之一，WHEN 启动服务，THEN 返回 `CONFIG_INVALID`；仅配置静态 token 时旧路径不变，仅配置成对动态变量时使用 login/refresh 会话。
- GIVEN 动态会话的 API 请求返回 401，WHEN refresh 成功，THEN 最多重试原请求一次并更新内存 access token 及可能轮换的 refresh cookie；WHEN refresh 失败，THEN 不递归 refresh、不循环重试。
- GIVEN 动态会话 WS 握手返回 401 且尚未发送 op，WHEN refresh 成功，THEN 只重连握手一次；WHEN 已发送 op 后断连或失败，THEN 不盲目重放 op。
- GIVEN 用户未批准删除，WHEN 客户端看到 `destructiveHint=true`，THEN 应保留人工批准；服务端不得通过改名或组合工具绕过。
- GIVEN 配置包含密码、静态或动态 access token、refresh cookie，WHEN 启动、调用、报错和退出，THEN stdout/stderr、reporter、错误摘要与诊断 URL 均不出现上述秘密明文；access token 仅按既有 Authorization header 与 WS 握手 query 契约传输，密码与 refresh cookie 不进入 URL；成功调用不产生 stderr 诊断行。

## MODIFIED — §6 追溯

| 需求 | 设计 | 场景 | 测试 |
|---|---|---|---|
| FR-S06-01～09 | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `core-S06-test-cases.md` |
| MCP 工具契约 | `mcp-tools.yaml` | 主时序及 §5.1～§5.4、§6 | UT-MCP-02～10、UT-MCP-41～49、ST-MCP-01～05、ST-MCP-18～21 |
| 四客户端配置 | 设计 §8 | §3.1 | UT-MCP-11、ST-MCP-06～09 |
| 整表 create、批量关系、48 KiB 建议与非原子边界 | 设计 §4.2、§4.4 | §5.1 | UT-MCP-41～44、ST-MCP-18 |
| 房间错误、动态 login/refresh、单次 retry/WS 重连与脱敏 | 设计 §4.3、§4.6、§5、§6 | §5.2、§5.3 | UT-MCP-45～48、ST-MCP-19～20 |
| 实际 ID、端点校验与物化保真 | 设计 §4.2、§4.7 | §5.4 | UT-MCP-49、UT-C-11、ST-MCP-21 |
