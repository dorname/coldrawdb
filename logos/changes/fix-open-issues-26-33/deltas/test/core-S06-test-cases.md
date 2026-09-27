# Delta — core-S06-test-cases.md（fix-open-issues-26-33 / #26）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：载荷软上限本地拦截与分批写入链路用例，对齐设计 §4.4 与需求 FR-S06-09。

## ADDED — UT 用例（追加）

| UT-MCP-23 | `update_diagram` 参数体积估算与软上限拦截 | 体积恰 ≤ 软上限 → 直写不拦截；体积 > 软上限 → 返回 `PAYLOAD_TOO_LARGE`，`isError=true`，details 含 `payload_bytes` / `soft_limit_bytes` / `suggestion` / `batch_hint`；**mock HTTP 断言零请求发出**；diagram 数据不变 |
| UT-MCP-24 | 软上限配置生效 | `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES` 覆盖默认值；非法值（< 16 KiB / 非整数）→ 按默认 256 KiB 处理并 warn（stderr 脱敏行）；retryable=false |
| UT-MCP-25 | `batch_hint` 构成建议 | 变更以表为主 → `suggested_batch=update_table`；以关系为主 → `update_reference`；纯布局 → `layout_diagram`；details 中 `tables` / `references` 计数与 diagram 实际一致 |

## ADDED — ST 用例（追加）

| ST-MCP-13 | 启动 stdio → 构造超软上限的大图变更 → `update_diagram` 被拦截 → 按指引分批（update_table × N → update_reference × M → layout_diagram），每批携带最新 `expected_revision` 串行 → get_diagram 验证 | 拦截响应结构正确；分批全部成功；revision 连续递增无跳号；最终 diagram 与目标变更一致 |
| ST-MCP-14 | 分批链路中人为制造一批 409（过期 expected_revision）→ 重试该批 | 失败批返回 `REVISION_CONFLICT`；已成功批次不回滚；以最新 revision 重试后该批成功；最终状态一致、无数据损坏 |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-23 | UT | update_diagram 软上限拦截 + PAYLOAD_TOO_LARGE 结构 + 零请求 |
| UT-MCP-24 | UT | 软上限环境变量配置生效与非法值回退 |
| UT-MCP-25 | UT | batch_hint 构成建议与计数 |
| ST-MCP-13 | ST | 超限拦截后按官方分批指引串行写成功 |
| ST-MCP-14 | ST | 分批链路单批 409 重试不损坏 revision |
