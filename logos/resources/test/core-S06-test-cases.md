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

> CI 若没有某个客户端二进制，ST-MCP-06～09 可由配置解析 + 官方 schema fixture 执行；至少 ST-MCP-01 必须使用真实 MCP client/Inspector 完成协议握手，不能全部以静态检查代替。

## 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、ST-MCP-03 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、ST-MCP-04/05 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、ST-MCP-06～09 |

## 5. Reporter 契约

每个测试完成后写一行 JSON：

```json
{"test_id":"UT-MCP-01","status":"PASS","duration_ms":12,"timestamp":"2026-08-18T00:00:00Z","module":"core","scenario":"S06"}
```

- `test_id` 必须在本文件登记且唯一。
- 失败写 `FAIL` 并可带脱敏 `message`，不得写 Token 或完整 diagram payload。
- 编排 runner 失败时也必须报告已执行步骤；不能因进程退出丢失整个结果账本。

