# [BUG] MCP 大图写入：单次载荷过大报错，需支持分批写入

## Describe the bug

通过 MCP（如 Cursor 中的 `user-coldrawdb`）对图表做**一次完整写回**时，若写入量较大（多表 + 多关系 + 布局/颜色等全量 `update_diagram`），工具调用会因**载荷过大**失败，无法一次写完。

实际排查中可见：助手侧会提示 `update_diagram` 载荷较大；若绕开 MCP 直接打后端 REST 虽可能绕过客户端限制，但不符合「必须走 MCP」的使用约束。当前细粒度工具（`update_table` / `update_field` / `update_reference` 等）虽可手工分批，但缺少官方约定与自动化分批策略，大图写入体验不稳定。

## To Reproduce

1. 打开一张表/关系数量较多的图（例如数十张表 + 数十条 FK 关系，并含布局/颜色等批量变更）
2. 在 Cursor（或其他 MCP 客户端）中调用 `update_diagram`，一次性传入完整 `diagram` + `expected_revision`
3. 观察：工具调用失败或被客户端拒绝，报「载荷太大」/ payload too large 一类错误
4. （对照）改用多次 `update_table` / `update_reference` 等小工具顺序写入，可绕过单次限额，但需自行处理 revision 串行与失败重试

## Expected behavior

大图写入应在**不绕开 MCP**的前提下可靠完成，建议至少其一：

1. **官方分批写入约定**：文档明确「单次 `update_diagram` 体积上限」；超限时引导使用 `update_table` / `update_field` / `update_reference` / `layout_diagram` 分批，并说明 revision 串行规则
2. **服务端/工具侧分批能力**（更优）：新增或增强 MCP 工具，支持按表/关系/区域分片写入（例如 `update_diagram` 接受 partial patch，或提供 `batch_update`），单次工具参数控制在客户端限额内
3. **错误可诊断**：超限时返回明确错误码与建议（当前体积、建议分片粒度），而不是模糊的「载荷太大」

## Screenshots

见会话截图：助手判断 `update_diagram` 载荷较大，拟改走后端 API 再 MCP 校验；后续讨论结论为应**分批写入且不能绕开 MCP**。

## Desktop

- OS: Windows / WSL2
- Client: Cursor MCP（`user-coldrawdb` / `coldrawdb-mcp` stdio）
- 相关工具：`update_diagram`、`update_table`、`update_reference`、`get_diagram`

## Additional context

- 触发场景：批量改表色、批量建 FK 关系、整图布局写回等「一次写大量变更」
- 约束：用户明确要求不能绕开 MCP 直接调 HTTP API
- 临时 workaround：按实体分批调用细粒度写工具，严格串行 `expected_revision`
- 影响面：S06 MCP；可能涉及 MCP 工具合同、客户端参数大小限制、以及后端是否需 partial update API
- 建议验收：
  1. 构造超过当前客户端限额的图变更，走 MCP 分批路径可完整写成功
  2. 单批失败时 revision 不损坏、可重试
  3. 文档或工具描述中可见分批指引 / 体积上限说明
