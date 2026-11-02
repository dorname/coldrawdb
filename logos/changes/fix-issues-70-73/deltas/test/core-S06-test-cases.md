## MODIFIED — UT-MCP-26 — 房间图经 op 通道写入

- **位置**：`mcp-server` 协作 diff / 会话逻辑。
- **GIVEN**：活跃房间 `diagramId` 匹配；基准表名与目标表名不同；假 socket 先回 `connected` 再回 `ack.serverRev`。
- **WHEN**：规划并提交房间写入。
- **THEN**：产生不含 fields 的 `table.update`；提交后 revision 为 ack 的 serverRev；`READ_ONLY` 在第一帧失败且不再继续发送；空 diff 保持当前 revision。
- **#71 修订**：无凭据的房间写回退不再断言笼统 `UNAUTHENTICATED`，而是在 GET rooms 前返回 `ROOMS_AUTH_REQUIRED`，details.reason 为 `TOKEN_MISSING`，详见 UT-MCP-45；普通非房间 API 401 仍保持 `UNAUTHENTICATED`。

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
