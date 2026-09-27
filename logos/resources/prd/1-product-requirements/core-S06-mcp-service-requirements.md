# S06：AI 客户端通过 MCP 管理数据库图表 — 产品需求

> 模块：core | 场景：S06 | 版本：V3 | 优先级：P1 | 提案：align-unified-prototype-and-add-mcp

## 1. Why：问题与目标

开发者已经能在 coldrawdb Web 编辑器中管理 ER 图，但在 Claude、Codex、Cursor、OpenCode 中设计或修改代码时，仍需离开当前工作流、手工复制 schema，再回到浏览器保存。目标是提供标准 MCP 服务，让四类 AI 客户端以一致的工具契约读取和管理图表，同时继续遵守 coldrawdb 的 revision、一致性与数据边界。

### 1.1 用户故事

- US-S06-01：作为开发者，我希望 AI 客户端列出并读取 coldrawdb 图表，以便基于真实 schema 回答和生成代码。
- US-S06-02：作为开发者，我希望 AI 客户端创建、导入和更新图表，并在覆盖新版本前显式处理 revision 冲突。
- US-S06-03：作为评审者，我希望从 AI 客户端导出 SQL、DBML 或 JSON，以便审阅拟议 schema。
- US-S06-04：作为管理员，我希望 MCP 服务不直接打开 SQLite、不记录凭据、不暴露任意 SQL，以便缩小自动化工具的权限面。
- US-S06-05：作为工具使用者，我希望 Claude、Codex、Cursor、OpenCode 使用同一个本地服务，而不需要维护四套业务逻辑。

## 2. What：功能需求

| ID | 需求 | 验收 |
|---|---|---|
| FR-S06-01 | 提供 MCP stdio 服务并完成 initialize、initialized、tools/list | 四客户端配置可启动服务；tools/list 返回七个稳定工具 |
| FR-S06-02 | 提供 `list_diagrams`、`get_diagram` | 返回结构化摘要或完整 diagram；不存在时返回可诊断错误 |
| FR-S06-03 | 提供 `create_diagram`、`update_diagram`、`delete_diagram`，以及画布写工具 | 非房间图 update 走 `PUT /api/v1/diagrams/{id}` 且必须携带 `expected_revision`；房间绑定图在 PUT 返回 `USE_OP_CHANNEL` 后改经 `/ws/rooms/{roomId}` 提交 op，成功响应仍为 `{id, revision}` |
| FR-S06-04 | 提供 `import_schema` | MVP 接受 drawdb JSON；SQL/DBML 文本导入仅在解析能力落地后开放对应枚举值 |
| FR-S06-05 | 提供 `export_schema` | 从完整 diagram 在 adapter 内确定性生成 JSON、DBML 或七类 SQL |
| FR-S06-06 | 提供四客户端配置 | Claude、Codex、Cursor、OpenCode 示例可复制，均指向同一二进制 |
| FR-S06-07 | 标注工具副作用 | 读工具 `readOnlyHint=true`；删除工具 `destructiveHint=true`；写工具不伪装为只读 |
| FR-S06-08 | 透传上游错误 | 网络、400/401/403/404/409/422/5xx 映射为稳定 MCP 错误码并保留安全的 request_id/details；房间写收编 409 的 `details.code=USE_OP_CHANNEL` 不得映射为 `REVISION_CONFLICT` |

## 3. 非功能需求

| ID | 约束 | 指标 |
|---|---|---|
| NFR-S06-01 | 启动性能 | 本地 release 二进制 initialize P95 ≤ 1s |
| NFR-S06-02 | 调用超时 | 默认 30s，可通过 `COLDRAWDB_REQUEST_TIMEOUT_SECS` 配置为 1～120s |
| NFR-S06-03 | 安全与日志 | stdout 只写 MCP 帧；成功 tool_call 不写 stderr；失败时 stderr 仅一行 JSON，含 `code` 与 `message`；Token、Authorization、Cookie 必须脱敏 |
| NFR-S06-04 | 数据边界 | 禁止 SQLite 文件访问、任意 SQL 执行和 shell 执行工具 |
| NFR-S06-05 | 兼容 | MCP 客户端协商版本，不硬编码拒绝未知的新版本；工具 schema 保持向后兼容 |
| NFR-S06-06 | 可测试 | 每个工具至少一个 UT；主链、409 和配置兼容具有 ST；全部写 OpenLogos reporter |

## 4. 范围与边界

### 4.1 MVP 范围

- 仅 stdio transport；进程由客户端启动和停止。
- 仅连接 `COLDRAWDB_BASE_URL` 指定的 HTTP 服务。
- `COLDRAWDB_ACCESS_TOKEN` 可选；配置后以 Bearer header 透传。
- `list_diagrams` 暂映射遗留只读端点 `GET /diagrams/queryAll`；其余 CRUD/导入映射 `/api/v1/diagrams*`。
- `export_schema` 先读取完整 diagram，再在 adapter 内进行纯函数序列化，不访问数据库。

### 4.2 已知现状约束

当前 `/api/v1/diagrams*` 尚未接入 S03 JWT 中间件，`GET /diagrams/queryAll` 也是遗留匿名端点。因此 MVP 的安全边界是“受信任本机进程 + 用户主动配置的 API 地址”，不能宣称已具备 diagram 级用户授权。远程 transport 与强制认证必须等 diagram API 完成权限接入后另案设计。

### 4.3 不在范围

- Streamable HTTP、SSE、OAuth、远程公网部署。
- 任意 SQL 执行、数据库探测、数据库文件路径参数。
- S03～S05 生产前端接入。
- MCP resources、prompts、sampling、elicitation；MVP 只提供 tools。

## 5. 验收场景

- GIVEN coldrawdb API 正常且客户端已配置 stdio 服务，WHEN 客户端完成 MCP 初始化，THEN 能发现七个工具及正确 annotations。
- GIVEN 已存在 diagram，WHEN 调用 `get_diagram` 与 `export_schema(format="dbml")`，THEN 返回同一 revision 对应的完整模型和确定性 DBML。
- GIVEN `expected_revision` 已过期，WHEN 调用 `update_diagram`，THEN 返回 `REVISION_CONFLICT` 和 `current_revision`，不自动覆盖。
- GIVEN diagram 绑定活跃协作房间且 `expected_revision` 等于当前物化 revision，WHEN 调用写工具，THEN adapter 经 WebSocket 提交 op 并返回新的 `revision`；viewer 得到 `READ_ONLY`；无 token 得到 `UNAUTHENTICATED` 且输出不含 token。
- GIVEN 用户未批准删除，WHEN 客户端看到 `destructiveHint=true`，THEN 应保留人工批准；服务端不得通过改名或组合工具绕过。
- GIVEN 配置包含 Token，WHEN 启动、调用、报错和退出，THEN stdout/stderr 与 reporter 均不出现 Token 明文；成功调用不产生 stderr 诊断行。

## 6. 追溯

| 需求 | 设计 | 场景 | 测试 |
|---|---|---|---|
| FR-S06-01～08 | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `core-S06-test-cases.md` |
| MCP 工具契约 | `mcp-tools.yaml` | §3～§5 | UT-MCP-02～10、ST-MCP-01～05 |
| 四客户端配置 | 设计 §8 | §3.1 | UT-MCP-11、ST-MCP-06～09 |

---

## 合并自 fix-open-issues-26-33（2026-09-27）

## ADDED — §2 功能需求（追加行）

| ID | 需求 | 验收 |
|---|---|---|
| FR-S06-09 | 提供大图分批写入官方约定与超限可诊断错误 | `update_diagram` 声明建议载荷软上限；adapter 发送前估算参数体积，超限返回 `PAYLOAD_TOO_LARGE`（details 含 `payload_bytes` / `soft_limit_bytes` / 建议分片粒度）；工具描述与文档写明分批指引（`update_table` / `update_field` / `update_reference` / `layout_diagram` 串行 `expected_revision`）；分批链路中单批失败 revision 不损坏、可重试 |

## ADDED — §5 验收场景（追加）

- GIVEN 一次 `update_diagram` 的序列化参数体积超过软上限，WHEN 客户端调用该工具，THEN 返回 `PAYLOAD_TOO_LARGE` 且 details 给出当前体积、上限与建议分片粒度，**不发送上游请求**、diagram 数据不变。
- GIVEN 一张超过软上限的大图变更，WHEN 客户端按分批指引以细粒度工具串行写入（每批携带最新 `expected_revision`），THEN 全部批次成功、最终 revision 连续递增、画布变更完整生效。
- GIVEN 分批链路中某一批失败（如 409 / 网络错误），WHEN 客户端修正后重试该批，THEN 已成功批次不回滚、失败批可基于当前 revision 重试，最终状态一致。
