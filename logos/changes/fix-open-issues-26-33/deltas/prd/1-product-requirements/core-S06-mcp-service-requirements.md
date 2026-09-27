# Delta — core-S06-mcp-service-requirements.md（fix-open-issues-26-33 / #26）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：大图写入不绕开 MCP 也可靠完成——官方分批写入约定 + 超限可诊断错误。

## ADDED — §2 功能需求（追加行）

| ID | 需求 | 验收 |
|---|---|---|
| FR-S06-09 | 提供大图分批写入官方约定与超限可诊断错误 | `update_diagram` 声明建议载荷软上限；adapter 发送前估算参数体积，超限返回 `PAYLOAD_TOO_LARGE`（details 含 `payload_bytes` / `soft_limit_bytes` / 建议分片粒度）；工具描述与文档写明分批指引（`update_table` / `update_field` / `update_reference` / `layout_diagram` 串行 `expected_revision`）；分批链路中单批失败 revision 不损坏、可重试 |

## ADDED — §5 验收场景（追加）

- GIVEN 一次 `update_diagram` 的序列化参数体积超过软上限，WHEN 客户端调用该工具，THEN 返回 `PAYLOAD_TOO_LARGE` 且 details 给出当前体积、上限与建议分片粒度，**不发送上游请求**、diagram 数据不变。
- GIVEN 一张超过软上限的大图变更，WHEN 客户端按分批指引以细粒度工具串行写入（每批携带最新 `expected_revision`），THEN 全部批次成功、最终 revision 连续递增、画布变更完整生效。
- GIVEN 分批链路中某一批失败（如 409 / 网络错误），WHEN 客户端修正后重试该批，THEN 已成功批次不回滚、失败批可基于当前 revision 重试，最终状态一致。
