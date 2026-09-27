# Delta — core-S06-mcp-service-design.md（fix-open-issues-26-33 / #26）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：大图分批写入官方约定（软上限 + 分批指引 + revision 串行规则）与超限可诊断错误。

## ADDED — §4.4 大图分批写入约定（fix-open-issues-26-33 / #26）

> 背景：MCP 客户端对单次工具调用参数体积有限制，大图全量 `update_diagram`（数十表 + 关系 + 布局/颜色）会触发「载荷太大」失败。本节约定不绕开 MCP 的可靠写入路径；MVP 不引入 partial update API 或新 batch 工具。

### 4.4.1 载荷软上限

- adapter 对 `update_diagram` 在**发送前**估算序列化参数体积（`{"id","expected_revision","diagram"}` JSON 字节数）。
- 软上限 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES`，默认 **256 KiB**（262144），可用环境变量覆盖；下限 16 KiB（防配置误伤）。
- 未超软上限：按既有路径直写，不拦截、不告警。
- 超软上限：**不发送上游请求**，返回 `PAYLOAD_TOO_LARGE`（见 §4.3 错误表新增行），details 至少含：
  - `payload_bytes`：本次估算体积
  - `soft_limit_bytes`：生效上限
  - `suggestion`：固定文案，引导分批（「按表/字段/关系分片：update_table / update_field / update_reference，布局用 layout_diagram；每批携带最新 expected_revision 串行执行」）
  - `batch_hint`：`{ "tables": <表数>, "references": <关系数>, "suggested_batch": "update_table|update_reference" }`（按变更构成给主建议）

### 4.4.2 分批指引与 revision 串行规则

- 分批写入 = 多次细粒度写工具调用，每批都是一次完整的「GET 最新 → 修改 → 带 `expected_revision` 写回」闭环（adapter 内部对细粒度工具已是此语义，见 §4.2）。
- **串行规则**：同一 diagram 的分批必须串行——第 N 批成功返回新 revision 后，第 N+1 批才能以该 revision 发起；禁止并行分批（会必然 409）。
- **失败语义**：任一批失败不影响已成功批次；失败批的 diagram 当前 revision 保持可读（`get_diagram` 取最新），修正参数后以最新 revision 重试该批即可；不存在「半批」状态（每批自身是一次原子 PUT/op 提交）。
- 房间绑定图同样适用：细粒度工具经 op 通道时 revision 语义不变（§4.2）。

### 4.4.3 工具描述外露

- `update_diagram` 的 description 必须写明软上限默认值与「超限请分批」指引；四个细粒度写工具的 description 标注其作为大图分批写入的官方分片手段。

## MODIFIED — §4.3 错误体验（错误表追加行）

| 条件 | MCP 错误码 | retryable | 用户动作 |
|---|---|---|---|
| 配置缺失/URL 非 http(s) | CONFIG_INVALID | false | 修正客户端配置并重启 |
| 连接失败/超时 | UPSTREAM_UNAVAILABLE / UPSTREAM_TIMEOUT | true | 启动 backend 或检查 URL |
| 400/422 | VALIDATION_ERROR | false | 根据 details 修正参数 |
| 401，或房间写缺少 token | UNAUTHENTICATED | false | 更新 Access Token |
| 403，或房间成员只读 | PERMISSION_DENIED / READ_ONLY | false | 使用有写权限的身份 |
| 404，或找不到绑定房间 | NOT_FOUND | false | 重新列出 diagram / 房间 |
| 409 且 details 含 current_revision | REVISION_CONFLICT | false | 读取最新 diagram，人工合并后重试 |
| 409 且 details.code=USE_OP_CHANNEL | 不作为最终错误；转入 op 通道 | false | 无需用户改参；通道失败时再返回对应码 |
| 5xx | UPSTREAM_ERROR | true | 稍后重试并使用 request_id 排查 |
| `update_diagram` 参数体积超软上限（本地拦截，未发请求） | PAYLOAD_TOO_LARGE | false | 按 details.suggestion 分批写入；或调大 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES`（自担客户端限制风险） |

错误结果使用 `isError: true`，structuredContent 至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`；不得包含 Authorization、Cookie、WebSocket URL 中的 token 或完整上游响应头。`PAYLOAD_TOO_LARGE` 属本地拦截，无 `request_id`。
