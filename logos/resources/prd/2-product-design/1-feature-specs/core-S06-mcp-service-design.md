# S06：MCP 服务能力 — 功能设计

> 模块：core | 场景：S06 | 版本：V3 | 优先级：P1
> 输入：`core-S06-mcp-service-requirements.md` | 时序：`core-S06-ai-client-mcp.md` | 契约：`mcp-tools.yaml`

## 1. 产品形态

S06 是无 GUI 的本地 adapter。Claude、Codex、Cursor 或 OpenCode 启动 `coldrawdb-mcp` 子进程，通过 stdin/stdout 交换 MCP JSON-RPC；adapter 使用 HTTP 调用已运行的 coldrawdb backend。stdout 严禁混入日志，所有诊断信息写 stderr。

## 2. 生命周期

| 状态 | 进入条件 | 允许行为 | 退出条件 |
|---|---|---|---|
| starting | 客户端创建进程 | 读取并校验环境变量，初始化 HTTP client | 配置有效→initializing；无效→退出码 2 |
| initializing | 收到 initialize | 协商协议版本，返回 serverInfo/capabilities/instructions | 收到 initialized→ready |
| ready | 初始化完成 | tools/list、tools/call | stdin EOF、SIGTERM→stopping |
| stopping | 客户端关闭 | 停止接收新调用，等待当前调用完成或超时 | 退出码 0 |
| failed | 协议/内部不可恢复错误 | stderr 输出脱敏诊断 | 非 0 退出 |

`instructions` 前 512 字符必须包含：服务只操作 `COLDRAWDB_BASE_URL`；写入前读取最新 revision；删除具有破坏性；不得把工具当作任意 SQL 通道。

## 3. 工具目录

| 工具 | 目的 | 上游/实现 | annotations |
|---|---|---|---|
| `list_diagrams` | 列出图表摘要 | `GET /diagrams/queryAll`（遗留只读） | readOnly=true, destructive=false, idempotent=true |
| `get_diagram` | 获取完整图表 | `GET /api/v1/diagrams/{id}` | readOnly=true, destructive=false, idempotent=true |
| `create_diagram` | 创建空图表 | `POST /api/v1/diagrams` | readOnly=false, destructive=false, idempotent=false |
| `update_diagram` | 全量保存并 revision 校验 | `PUT /api/v1/diagrams/{id}` | readOnly=false, destructive=false, idempotent=false |
| `delete_diagram` | 软删除图表 | `DELETE /api/v1/diagrams/{id}` | readOnly=false, destructive=true, idempotent=true |
| `import_schema` | 导入 drawdb JSON 并重键 | `POST /api/v1/diagrams/import` | readOnly=false, destructive=false, idempotent=false |
| `export_schema` | 导出 JSON/DBML/SQL | GET 完整图表 + adapter 纯函数序列化 | readOnly=true, destructive=false, idempotent=true |
| `update_table` | 更新表属性，或一次创建一张含完整 fields 的表 | GET + 非房间 PUT / 房间 op；create 返回实际 ID 映射 | readOnly=false, destructive=false, idempotent=false |
| `update_field` | 修改字段属性与字典绑定 | GET + 非房间 PUT / 房间 op | readOnly=false, destructive=false, idempotent=false |
| `update_reference` | 平铺单条关系 create/update/delete，或数组批量 create | GET + 非房间 PUT / 房间逐 op；统一规范化与端点校验 | readOnly=false, destructive=false, idempotent=false |
| `layout_diagram` | 自动布局 | GET + adapter 布局 + 非房间 PUT / 房间 op | readOnly=false, destructive=false, idempotent=true |
| `update_dictionary` | 管理数据字典 | GET + 非房间 PUT / 房间 op | readOnly=false, destructive=true, idempotent=false |

工具总数保持 12。`update_table` 与 `update_reference` 只扩展既有工具，不新增 batch 工具。

## 4. 交互原则

### 4.1 读调用

- `list_diagrams` 默认最多返回 100 条摘要；adapter 负责稳定排序和本地 `limit` 截断。
- `GET /diagrams/queryAll` 必须读取并返回每张图的真实 `revision` 与 `updated_at`，同时保留 `lastModified` 兼容字段和 `is_deleted=0` 过滤。旧数据 `updated_at` 缺失时可回退到 `lastModified`；不得用 `0/null` 覆盖已有值，也不得用当前时间伪造修改时间。
- MCP `list_diagrams` 的每个摘要项必须包含 `revision`、`updated_at` 键并透传上述真实值；现有名称过滤、排序和 limit 行为不变。本变更不增加 table/reference 计数。
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

### 4.2 写调用

### 4.2.1 通用写入与 revision

- `create_diagram` 只接受 `name` 和可选 `database`，与实际后端 `CreateReq` 对齐；复杂初始模型使用 `import_schema`，或使用本节的整表/关系分片。
- `update_diagram` 必须显式提供 `expected_revision` 与完整 `diagram`，adapter 不自动读取后覆盖。
- 非房间图的 `update_diagram`、`update_table`、`update_field`、`update_reference`、`layout_diagram`、`update_dictionary` 仍经 `PUT /api/v1/diagrams/{id}` 写回。
- 房间绑定图的上述写工具在 PUT 返回 409 且 `details.code=USE_OP_CHANNEL` 时，不得把该响应当成最终结果。adapter 使用 `GET /api/v1/rooms` 查找 `diagramId` 匹配且未归档的房间，以当前物化文档为基准对目标文档做 diff，再连接 `ws(s)://…/ws/rooms/{roomId}?token=` 按序提交 op，直到收到 ack。成功响应保持各工具既有瘦响应，整表 create 另含实际 ID 映射。
- 房间写入前若物化 `revision` 与 `expected_revision` 不一致，返回 `REVISION_CONFLICT`，不提交 op。无差异则保持当前 revision，不连接后空写。
- 同一 diagram 的细粒度写调用严格串行；后一调用使用前一调用返回的最新 revision。禁止通过并行分片绕过乐观锁。
- `delete_diagram` 必须显式提供 `confirm: true`；这不能替代客户端审批，只是服务侧第二道防误触。
- `import_schema` MVP 仅接受 `format: "drawdb_json"`，传给 API 的 payload 必须为 JSON object。导入始终新建并重键，调用方后续通过 `get_diagram` 获取实际 ID。

### 4.2.2 `update_table`：整表 create 与兼容 update

- 新增可选 `action=create|update`，缺省为 `update`。旧调用不增加任何必填参数。
- `action=create` 沿用平铺表参数：`table_id` 作为调用方原始表 ID，`name` 与完整 `fields` 数组必填，comment/x/y/color/locked 等表属性可选；一次调用只追加一张表。
- create 在本地构造目标快照时采用与导入一致的冲突安全重键：为 table 及每个 field 无条件生成新的 `auto-` 全局唯一 ID，不因输入 ID 暂未冲突而原样保留。字段顺序、字段属性和表属性保持完整，不在 create 中隐式创建 references。
- create 成功响应为 `{id, revision, table_id, field_id_map}`：`id` 是 diagram ID，`table_id` 是服务端实际表 ID，`field_id_map` 是调用方旧 field ID 到服务端实际 field ID 的完整映射。后续 `update_reference` 必须使用这些实际值；响应不返回整图。
- `action=update` 保持既有平铺表属性参数及 `{id, revision}` 响应，按 `table_id` 修改单表；update 不接受 `fields`，不得借 update 替换整表字段数组。
- 非房间 create 通过一次全量快照 PUT 追加；房间 create 对应一个携带完整 fields 的 `table.create` op。响应中的实际 ID 必须与最终写入/ack 后读取的实体一致。

### 4.2.3 `update_reference`：单条/批量互斥与规范化

- 既有平铺 `action=create|update|delete` 保持不变。仅 `action=create` 可改用 `references` 数组，一次提交多条关系；数组每项使用与平铺 create 相同的关系字段。
- 批量形态的顶层只允许 diagram `id`、`action=create` 与 `references`；`references` 不得与 `ref_id`、`start_table_id`、`start_field_id`、`end_table_id`、`end_field_id`、`cardinality`、`on_delete`、`on_update`、`name`、`color` 等任一平铺关系字段同时出现。update/delete 携带 `references`，或 create 携带空数组，均在写前返回 `VALIDATION_ERROR`。
- adapter 先完整校验所有数组项，再基于同一最新快照构造全部关系；任一项失败则整批不发送 PUT/WS，revision 不变。不得先写入合法前缀再报告后续项失败。
- 非房间合法批次经一次 GET/PUT 快照写回追加全部关系。房间合法批次仍以一次目标快照计算 diff，但可产生多个 `reference.create` op，逐条等待 ack；其非原子边界见 §4.4.3。
- 平铺单条与数组批量必须调用同一关系构造/规范化逻辑：外部小写 `cardinality` 写入后端快照的 `type_`，不得误写为后端不消费的 `cardinality`；`on_delete` / `on_update` 显式由 `cascade|set_null|set_default|restrict|no_action` 转为 `CASCADE|SET_NULL|SET_DEFAULT|RESTRICT|NO_ACTION`。
- name、color、端点和其他字段在单条与批量路径使用同一默认值和校验规则；成功响应仍为 `{id, revision}`，不在批量响应中回显整图。

### 4.2.4 实际 ID 与写前端点校验

- 单个和批量 `action=create` 都必须在当前完整 diagram 快照中校验：`start_table_id`、`end_table_id` 指向存在的表，`start_field_id`、`end_field_id` 指向存在的字段，且 start field 属于 start table、end field 属于 end table。
- 导入前旧 ID、整表 create 前旧 ID、任意悬空 ID、表存在但字段不归属，均返回 `VALIDATION_ERROR`。错误在 PUT/WS 前产生，单条与批量均为零写入；批量错误可携带安全的数组 index 与端点类别用于定位。
- adapter 不保留或猜测跨调用的旧→新映射，不在关系 create 中按名称猜测端点，也不将无效端点降级为可后续修复的关系。导入后使用 `get_diagram`，整表 create 后使用响应映射取得实际 ID。
- 本规则只新增 create 的防损坏校验；既有 update/delete 参数与有效 create 调用行为不变。

### 4.3 错误体验

| 条件 | MCP 错误码 | retryable | 用户动作 |
|---|---|---|---|
| 配置缺失/URL 非 http(s)，或静态 token 与动态登录混配、email/password 未成对 | CONFIG_INVALID | false | 修正客户端配置并重启 |
| HTTP 连接失败 | UPSTREAM_UNREACHABLE | true | 按 message/details 中的 BASE_URL origin 启动 backend 或修正地址 |
| 请求超时 | UPSTREAM_TIMEOUT | true | 检查上游负载或网络后重试 |
| 400/422、本地 schema/互斥/端点校验失败 | VALIDATION_ERROR | false | 根据 details 修正参数；端点错误时改用服务端实际 ID |
| 普通 API 401，或动态 login/refresh 最终失败 | UNAUTHENTICATED | false | 修正静态 token 或动态登录凭据 |
| 房间写回退无凭据 | ROOMS_AUTH_REQUIRED，details.reason=TOKEN_MISSING | false | 配置静态 token，或成对配置动态登录凭据 |
| 房间写回退 rooms/WS 最终 401 | ROOMS_AUTH_REQUIRED，details.reason=TOKEN_INVALID_OR_EXPIRED | false | 更新凭据；动态模式确认 login/refresh 可用 |
| 403，或房间成员只读 | PERMISSION_DENIED / READ_ONLY | false | 使用有写权限的身份；不得改写为 ROOMS_AUTH_REQUIRED |
| 404，或找不到绑定房间 | NOT_FOUND | false | 重新列出 diagram / 房间 |
| 409 且 details 含 current_revision | REVISION_CONFLICT | false | 读取最新 diagram，人工合并后重试 |
| 409 且 details.code=USE_OP_CHANNEL | 不作为最终错误；转入 op 通道 | false | 无需用户改参；通道失败时再返回对应码 |
| 5xx | UPSTREAM_ERROR | true | 稍后重试并使用 request_id 排查；不得混淆为凭据错误 |
| `update_diagram` 参数体积超软上限（本地拦截，未发请求） | PAYLOAD_TOO_LARGE | false | 按 details.suggestion 以整表和关系数组分片；或调大软上限（自担客户端限制风险） |

工具错误结果使用 `isError: true`，`content[0].text` 为完整 `ToolError` 的 JSON 序列化，至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`。HTTP 连接失败的 message 与 `details.base_url` 均包含实际 BASE_URL 的脱敏 origin（scheme/host/port，不含 userinfo、query、fragment 或 token）。

`ROOMS_AUTH_REQUIRED` 只在统一 `USE_OP_CHANNEL` 回退点上下文化生成；普通非房间 401 仍为 `UNAUTHENTICATED`。rooms 401 的安全上游诊断继续包含 `http_status`、`content_type`、截断且脱敏的 `body_excerpt` 和可用 `request_id`；本地 `TOKEN_MISSING` 未发 rooms 请求，不伪造这些上游字段。

错误结果不得包含 `structuredContent`：各工具的 `outputSchema` 只约束成功响应。成功结果仍保持 `structuredContent` 与 `content[0].text` 表示同一对象，协议不变。

所有错误文本、诊断 URL 和 details 均不得包含密码、Authorization、access token、refresh cookie、WebSocket query token 或完整上游响应头；实际 WS 握手仍可按既有 query 契约传输 access token，但日志和诊断 URL 必须删除该秘密。即使上游 body 回显秘密，body 摘要也必须先脱敏再截断。

### 4.4 大图分批写入约定

### 4.4.1 载荷软上限

- adapter 对 `update_diagram` 在发送前估算序列化参数体积（`{"id","expected_revision","diagram"}` JSON 字节数）。
- 软上限 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES`，默认 256 KiB（262144），可用环境变量覆盖；下限 16 KiB（防配置误伤）。
- 未超软上限：按既有路径直写，不拦截、不告警。
- 超软上限：不发送上游请求，返回 `PAYLOAD_TOO_LARGE`，details 至少含 `payload_bytes`、`soft_limit_bytes`、`suggestion`，以及 `{tables, references, suggested_batch}` 形式的 `batch_hint`。
- 既有软上限不是 MCP 客户端统一硬限制，也不保证小于每个客户端的实际传输边界。

### 4.4.2 48 KiB 保守建议与分片顺序

- 工具描述和四客户端文档建议：整表 `update_table(action=create)` 一次一张表，随后用 `update_reference(action=create, references=[...])` 按关系数组分片；单次工具参数保守控制在约 48 KiB。
- 48 KiB 是降低客户端差异风险的预算建议，不是新的 adapter 拒绝阈值，不修改 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES` 的 256 KiB 默认值。
- 约 63 KB 仅描述特定客户端的实测边界，不得写成协议限制、所有客户端共同限制或精确保证。
- 推荐顺序为：创建空图或导入新图 → 逐表 create 并保存每次实际 ID 映射 → 以实际 ID 构造关系并分批 create → 必要时布局/更新附属对象。禁止为了让旧 ID 可用而跳过映射。

### 4.4.3 revision 串行与非原子边界

- 每一细粒度调用都是一次完整的「GET 最新 → 校验并修改 → 携带 `expected_revision` 写回」闭环；第 N 批成功后，第 N+1 批才能使用其最新 revision，禁止并行分片。
- 单次非房间整表 create 或批量关系 create 各自对应一次快照 PUT；单次房间整表 create 对应一个 op；单次房间批量关系 create 可对应多个 op 并逐条 ack。
- 跨工具调用没有事务或补偿回滚。后续批次失败不会撤销已成功的表或关系；调用方读取最新 revision、修正失败批后重试，不得假设整张大图一次原子生效。
- 房间批量关系不宣称跨 op 原子、不宣称 revision 只增加 1；若中途失败，已 ack 的 op 保留，未发送或未 ack 的项不得盲目重放。非房间单次 PUT 的快照原子性也不扩展为跨调用原子性。

### 4.4.4 工具描述外露

- `update_diagram` description 保留软上限默认值与超限分批指引。
- `update_table` description 标明 create/update 两种 action、整表分片和实际 ID 映射输出。
- `update_reference` description 标明单条/批量互斥、按数组控制约 48 KiB、共用 `type_`/级联规范化及实际 ID 要求。
- `update_field`、`layout_diagram` 继续标为细粒度分片手段；工具总数不增加。

### §4.5 update_diagram 错误可诊断与房间图回退鲁棒性（fix-issues-42-44-mcp-and-relation-hit / #42）

**背景**：#42——`ApiClient::request` 对非 JSON 响应体一律吞成「上游返回非 JSON 响应」，丢失 status/Content-Type/body；且 409 `USE_OP_CHANNEL` 识别要求 body 为特定 JSON 结构（`details.code`），body 非 JSON 时房间图 WS op 通道回退不触发，用户被迫只能 `import_schema` 新建。

合同条款：

- **R-MCPERR-01 错误可诊断**：上游非 2xx 时，无论 body 是否为合法 JSON，`ToolError.details` 必须携带：
  - `http_status`（整数）
  - `content_type`（响应 Content-Type，缺省为空串）
  - `body_excerpt`（body 文本截断前 200 字符；body 为合法 JSON 时为原 JSON 摘要）
  错误 message 不再只有「非 JSON」；网关 502/504 HTML 页、代理拦截页必须可被用户从错误中直接辨认。
- **R-MCPERR-02 USE_OP_CHANNEL 鲁棒识别**：409 的识别路径依次为——①body 为 JSON 且 `details.code == "USE_OP_CHANNEL"`（既有）；②body 非 JSON 但文本包含 `USE_OP_CHANNEL` 字样。任一路径命中即走 WS op 通道回退（`write_room_diagram`），不得落入笼统 VALIDATION_ERROR。
- **R-MCPERR-03 op 通道失败归因**：op 通道写入失败时错误码必须为协作通道语义（`UPSTREAM_UNAVAILABLE` / 既有 collab 错误码），message 指明「协作通道」而非「非 JSON」。
- **R-MCPERR-04 import_schema 仅新建文案**：`import_schema` tool 描述明确「仅创建新图表，不覆盖已有 id；覆盖/更新已有图表请用 update_diagram / update_table 等增量工具」。

验收：UT-MCP-36 / UT-MCP-37 / UT-MCP-38。

### 4.6 房间凭据会话与单次恢复（fix-issues-70-73 / #71）

### 4.6.1 凭据模式与内存会话

- **静态模式**：仅配置 `COLDRAWDB_ACCESS_TOKEN`，沿用既有 Bearer 与 WS token 注入；不伪造 refresh cookie，不对静态 token 调 refresh。
- **动态模式**：仅配置成对的 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD`。首次需要鉴权时调用 `POST /api/v1/auth/login`，从响应内存保存 access token 与 HttpOnly refresh cookie；refresh 响应如轮换 cookie，必须原子替换内存值。
- **匿名模式**：两者均未配置，可继续调用无需鉴权的 API；进入房间写回退时按 `TOKEN_MISSING` 失败。
- 静态 token 与任一动态变量同时出现，或 email/password 只出现一项，启动即 `CONFIG_INVALID`。不存在凭据优先级或隐式回退到另一种模式。
- login/refresh 仅复用 S03 既有端点和 Cookie 契约；refresh token 不要求用户复制，不持久化，不新增 service token 或延长 JWT TTL。

### 4.6.2 API 401 的单次 refresh/retry

- 仅动态模式在业务 API 401 时调用一次 `POST /api/v1/auth/refresh`；成功后以新 access token 重试原请求最多一次，并保存可能轮换的 refresh cookie。
- refresh 自身 401/失败不得再次触发 refresh，业务请求重试后的第二个 401 也不得开启新循环。最终错误按普通 API 或房间回退上下文映射。
- 403、5xx、超时、连接失败和 revision conflict 不套用该 401 恢复逻辑；除本条款明确的原请求一次重试外，不增加写操作自动重试。

### 4.6.3 统一房间回退错误上下文化

- PUT 命中 `USE_OP_CHANNEL` 后，匿名模式直接返回 `ROOMS_AUTH_REQUIRED` + `details.reason=TOKEN_MISSING`，不先发送必失败的 `GET /api/v1/rooms`。
- 静态或动态模式按既有路径访问 rooms；动态模式可先经 §4.6.2 恢复。rooms 最终 401、或动态 login/refresh 最终无法提供有效凭据时，返回 `ROOMS_AUTH_REQUIRED` + `details.reason=TOKEN_INVALID_OR_EXPIRED`。
- rooms 403 仍为 `PERMISSION_DENIED` / `READ_ONLY`，5xx 仍为 `UPSTREAM_ERROR`，网络/超时仍按协作通道语义；不得笼统改写成凭据问题。

### 4.6.4 WS 握手重连与 op 不重放

- 仅动态模式、仅在 WS 握手返回 401 且本连接尚未发送任何 op 时，允许 refresh 一次并重新握手一次。新握手使用更新后的 access token。
- 第二次握手 401 按 `ROOMS_AUTH_REQUIRED` / `TOKEN_INVALID_OR_EXPIRED` 返回；refresh 不递归。
- 已发送 op 后的断连、ack 超时或服务端 error 不触发自动重连重放；调用方必须重新读取物化 revision 和关系状态后决定后续操作。

### 4.7 导入重键后的关系一致性与物化保真（fix-issues-70-73 / #72）

- ID 重写只发生在既有导入规范化与本 delta 的整表 create；调用方必须把响应或重新读取得到的实际 ID 作为后续唯一事实来源。
- 关系 create 的端点校验在 adapter 取得当前完整快照后、任何 PUT/WS 前执行；单个/批量共享 §4.2.4 规则，批量任一项无效则整批零写入。
- 房间首次物化仅原样加载存量非空图，不再次生成 table/field/reference/area/note ID，不重写 references 端点，不丢弃或重建 areas/notes。
- 不在物化阶段恢复旧 ID、不按名称匹配、不猜测已污染数据的历史映射；已有悬空关系的启发式修复不在本提案范围。
- 真实验收链路必须为「导入 → 读取实际 ID → 建关系 → 开房间 → 首 op → 再读取」，并核对实体 ID、关系端点、areas 与 notes 保真。

## 5. 配置

| 环境变量 | 必填 | 默认 | 校验 |
|---|---|---|---|
| `COLDRAWDB_BASE_URL` | 是 | 无 | http/https；移除尾部 `/`；禁止 userinfo |
| `COLDRAWDB_ACCESS_TOKEN` | 否 | 无 | 静态凭据；仅内存使用，不输出、不写文件；与任一动态登录变量互斥 |
| `COLDRAWDB_AUTH_EMAIL` | 否 | 无 | 动态登录账号；必须与 `COLDRAWDB_AUTH_PASSWORD` 成对出现；不得与静态 token 混配 |
| `COLDRAWDB_AUTH_PASSWORD` | 否 | 无 | 动态登录密码；secret；仅用于固定 login 请求体，不进入 URL、日志或 reporter |
| `COLDRAWDB_REQUEST_TIMEOUT_SECS` | 否 | 30 | 1～120 的整数；适用于 login/refresh 与业务请求 |
| `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES` | 否 | 262144 | 不低于 16384 的整数；仅控制 `update_diagram` 本地软上限，不是 48 KiB 分片建议的硬阈值 |
| `RUST_LOG` | 否 | warn | 日志写 stderr；敏感字段脱敏 |

## 6. 安全设计

- adapter 不依赖 sqlite/SeaORM，不接受数据库路径。
- HTTP client 只拼接契约中的固定路径；diagram id 作为单一 path segment 编码。动态凭据只访问固定 login/refresh 路径，不改变 S03 Cookie 契约。
- 不提供 URL、header、method 等“通用 HTTP”工具参数。
- 密码只进入 login 请求体；access token 只进入 Authorization header 及既有 WS 握手 query；refresh cookie 只进入 refresh 请求 Cookie。实际 WS 连接可以使用既有 query 契约，但任何日志、错误、reporter 或诊断 URL 必须删除该 query 秘密。
- access token、refresh cookie 与可能的 cookie 轮换值仅保存在进程内存；静态 token、密码均不写文件，不新增持久化凭据存储。
- Debug、Error、body_excerpt、reporter 均使用脱敏包装类型；上游响应若回显密码、Authorization、Cookie 或 token，也必须先脱敏再摘要。
- stdio transport 不监听端口；远程主机不得通过网络直接访问 MCP 进程。
- 动态会话不扩大 diagram/rooms 权限，不改变后端鉴权开关，不以可选凭据暗示匿名用户获得授权。

## 7. 可观测性

成功的 `tools/call` 不写 stderr。失败时 stderr 恰好一行 JSON，字段为 `event=tool_call`、`tool`、`duration_ms`、`status=error`、`code`、`message`。禁止记录 diagram 正文、密码、Authorization、access token、refresh cookie、WebSocket query token 或完整上游响应头；body_excerpt 只保留脱敏后的安全诊断。stdout 仅允许 MCP 帧。

login、refresh、原请求重试和 WS 重连可以记录不含秘密的事件类别、次数与最终状态，但不得记录请求体、Cookie、token 或带 token 的 URL。OpenLogos reporter 只记录用例结果与安全错误码，不记录任何凭据值。

## 8. 四客户端配置

以下示例中的 `/ABS/PATH/coldrawdb-mcp` 必须替换为绝对路径，Token 推荐由客户端允许的环境变量转发机制提供。

### 8.1 Claude Code（项目 `.mcp.json`）

```json
{
  "mcpServers": {
    "coldrawdb": {
      "type": "stdio",
      "command": "/ABS/PATH/coldrawdb-mcp",
      "args": [],
      "env": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

等价 CLI：`claude mcp add --transport stdio --env COLDRAWDB_BASE_URL=http://localhost:3000 coldrawdb -- /ABS/PATH/coldrawdb-mcp`。

### 8.2 Codex（项目 `.codex/config.toml` 或用户配置）

```toml
[mcp_servers.coldrawdb]
command = "/ABS/PATH/coldrawdb-mcp"
args = []
env = { COLDRAWDB_BASE_URL = "http://localhost:3000" }
startup_timeout_sec = 10
tool_timeout_sec = 30
default_tools_approval_mode = "writes"
```

等价 CLI：`codex mcp add coldrawdb --env COLDRAWDB_BASE_URL=http://localhost:3000 -- /ABS/PATH/coldrawdb-mcp`。配置格式依据 OpenAI 官方 MCP 文档：<https://developers.openai.com/codex/mcp.md>。

### 8.3 Cursor（项目 `.cursor/mcp.json`）

```json
{
  "mcpServers": {
    "coldrawdb": {
      "type": "stdio",
      "command": "/ABS/PATH/coldrawdb-mcp",
      "args": [],
      "env": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

### 8.4 OpenCode（`opencode.json`）

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "coldrawdb": {
      "type": "local",
      "command": ["/ABS/PATH/coldrawdb-mcp"],
      "enabled": true,
      "environment": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

## 9. 官方兼容性依据（核验日期：2026-08-18）

- Claude Code：<https://code.claude.com/docs/en/mcp.md>（stdio/HTTP，项目 `.mcp.json`）
- Codex：<https://developers.openai.com/codex/mcp.md>（stdio/Streamable HTTP，`config.toml`）
- Cursor：<https://cursor.com/docs/mcp.md>（stdio/SSE/Streamable HTTP，`.cursor/mcp.json`）
- OpenCode：<https://opencode.ai/docs/mcp-servers/>（`type: local`、command 数组、environment）
