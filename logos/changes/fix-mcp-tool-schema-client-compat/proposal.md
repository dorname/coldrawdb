# 变更提案：fix-mcp-tool-schema-client-compat

> module: core | created: 2026-11-03

## 变更原因

fix-issues-70-73 为 `update_table` / `update_reference` 的 inputSchema 引入了
`allOf + if/then/else`（含 `update_reference` 的 `oneOf + not/anyOf` 嵌套互斥）等高级
JSON Schema 条件结构。MCP 服务端把它们原样放进 `tools/list` 响应后，Claude Code
客户端在工具转换/校验阶段直接丢弃这两个工具：所有重连后的窗口只暴露 10 个工具，
`update_table` / `update_reference` 从客户端工具清单消失，#70 大图分片写回路径
（dspm 会话实测）被阻断。11-03 14:57 重建的二进制是首个包含该 schema 的产物，
两个窗口（dspm 与 coldrawdb）重连后均复现 10 工具。

## 变更类型

接口级变更（MCP 工具契约的 schema 表现层；工具行为、参数、校验与错误语义均不变）

## 变更范围

- 影响的需求文档：无（S06 需求的工具能力清单不变，仍为 12 工具）
- 影响的功能规格：`logos/resources/api/mcp-tools.yaml` — `update_table` 与
  `update_reference` 的 `inputSchema` 拍平为普通 object schema，条件规则改写进
  字段 description；新增「客户端兼容性约束」条款（inputSchema/outputSchema 顶层
  禁用 `allOf` / `oneOf` / `anyOf` / `if` / `then` / `else` / `not`）
- 影响的业务场景：S06（MCP stdio 服务；不改变任何编排时序，仅工具清单可见性修复）
- 影响的 API：MCP 工具 `update_table`、`update_reference`（schema 表现层；
  upstream HTTP 端点与语义不变）
- 影响的 DB 表：无
- 影响的编排测试：无（MCP 无编排测试）
- 影响的测试用例：`logos/resources/test/core-S06-test-cases.md` 新增 UT-MCP-50
  （全工具 schema 客户端兼容守卫）

## 部署影响

- 是否需要部署：否
- 部署原因：MCP 为本地 stdio 服务，无独立部署环境；交付含 release 二进制重建
  （schema 在编译期经 `include_str!` 内嵌），客户端重连即生效
- 影响环境：无（本地）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 拍平 `mcp-tools.yaml` 中 `update_table` 与 `update_reference` 的 `inputSchema`：
   删除 `allOf/if/then/else/oneOf/not/anyOf` 条件结构，「create 必填 name+fields、
   update 禁止 fields」「批量 references 仅限 create 且与平铺字段互斥」等条件规则
   移入对应字段与工具的 description。这些条件在 `tools/call` 时本就由服务端逐一
   强校验（违反返回 `VALIDATION_ERROR` 且零写入），schema 条件结构仅是提示，
   拍平不改变任何校验行为。
2. 在契约中登记客户端兼容性约束：工具 `inputSchema`/`outputSchema` 顶层仅允许
   `type`/`properties`/`required`/`additionalProperties`/`items`/`enum` 等基础关键字，
   条件语义一律走 description + 服务端校验。
3. 新增 UT-MCP-50 契约守卫测试：遍历 `McpService::tools()` 全部工具，断言
   inputSchema/outputSchema 顶层不含高级条件关键字，且 `update_table`/`update_reference`
   关键属性仍在（防止拍平时丢字段）。
4. 重建 release 二进制并做真实 stdio `tools/list` 探针复核 12 工具全部可见。
