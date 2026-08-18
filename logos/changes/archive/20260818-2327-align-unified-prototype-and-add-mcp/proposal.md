# 变更提案：align-unified-prototype-and-add-mcp

> module: core | created: 2026-08-18

## 变更原因（Why）

统一主原型 `core-01-editor-prototype.html` 已覆盖 S01～S05，但资源索引、场景总览、场景规格元数据和实现清单仍混用旧独立原型及历史实现状态，导致“原型能力、生产前端能力、后端能力”三者无法准确追溯。现状审计确认：统一主原型 ST-PU-01～ST-PU-18 全部通过，既有 ST-PU-19 自动回归通过；后端 S03～S05 API/DB/WS 与测试已实现，生产前端尚未接入；`core-03/04/05-*-prototype.html` 仅是历史参考，且存在未绑定控件与重复测试锚点，不应继续作为现行验收入口。

同时，coldrawdb 目前只能通过浏览器或 REST/WS 接口使用，无法被 Claude、Codex、Cursor、OpenCode 等 AI 编程客户端以标准工具协议发现和调用。需要新增遵循 MCP 规范的适配服务，在不绕过现有鉴权、权限、revision 和领域校验的前提下开放图表管理、导入和导出能力。

## 变更类型

需求级 + 设计级 + 接口级 + 代码级 + 测试级

## 变更范围（What）

- 影响的需求文档：
  - 更新 core 场景总览和需求追溯，新增全局场景 S06“AI 客户端通过 MCP 管理数据库图表”
  - 将 `scenario_counter.next_id` 从 6 更新为 7
- 影响的功能规格：
  - 将 `core-01-editor-prototype.html` 明确为 S01～S05 唯一现行主原型
  - 将 `core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html` 明确标记为历史参考，不修补其交互、不作为验收入口
  - 校正 S03/S04/S05 规格顶部原型引用、操作指南与生产实现状态声明
  - 新增 S06 MCP 服务功能规格、安全边界和四客户端接入说明
- 影响的业务场景：
  - 校正 S03/S04/S05 的真实状态：后端已实现，生产前端未接入，统一原型仅为模拟演示
  - 新增 S06 场景时序图，并由时序图推导 MCP tool contract
- 影响的 API：
  - 不修改现有 REST/WS 端点语义
  - 新增 MCP 工具契约：`list_diagrams`、`get_diagram`、`create_diagram`、`update_diagram`、`delete_diagram`、`import_schema`、`export_schema`
  - MCP 服务通过现有 coldrawdb REST API 调用业务能力，禁止直连 SQLite 或暴露任意 SQL 执行
- 影响的 DB 表：无
- 影响的编排测试：
  - 固化 ST-PU-01～ST-PU-19 为可重复执行的统一原型浏览器回归
  - 新增 S06 MCP 初始化、工具发现、读写链路、revision 冲突、错误映射和四客户端配置兼容测试
  - 所有新增自动化测试写入 OpenLogos reporter
- 影响的实现与文档：
  - 新增独立 Rust MCP adapter/service，MVP 使用 stdio transport
  - 新增 Claude、Codex、Cursor、OpenCode 四套配置示例
  - 更新资源索引、架构说明、实现清单、README、AGENTS/CLAUDE 项目状态与统计口径

## 明确边界

- 本变更不继续维护三个历史独立原型；其已发现的问题通过取消现行入口和补充历史标识消除误导。
- 本变更不实现 S03～S05 生产前端。该缺口会在规格和实现清单中如实登记，并建议后续以独立 OpenLogos 变更完成登录、房间与实时协作前端接入。
- MVP 仅承诺 stdio transport，因为四个目标客户端均支持；Streamable HTTP、Bearer/OAuth 远程接入作为后续可选增量，不纳入本次验收。
- MCP 写工具继续受客户端人工批准、coldrawdb 权限与 revision 乐观锁约束；不得提供绕过业务 API 的数据库文件访问或任意 SQL 能力。

## 验收标准

- **ALIGN-AC-01 原型入口唯一**：资源索引、S03/S04/S05 规格和实现清单一致指向 `core-01-editor-prototype.html`，旧原型均标记为历史参考。
- **ALIGN-AC-02 状态真实**：文档一致说明 S03～S05 后端已实现、生产前端未接入、主原型为本地模拟，不再出现“后端待实现”或“多个现行主原型”的冲突表述。
- **ALIGN-AC-03 原型完整性**：ST-PU-01～ST-PU-19 可由仓库内脚本重复执行并全部通过；结果写入 OpenLogos reporter。
- **MCP-AC-01 协议互通**：服务可完成 MCP 初始化与 tools/list，Claude、Codex、Cursor、OpenCode 的 stdio 配置通过结构校验和至少一套真实 MCP 客户端/Inspector 冒烟。
- **MCP-AC-02 读链路**：客户端可列出并读取图表，可按支持格式导出 schema，返回结构化结果和可诊断错误。
- **MCP-AC-03 写链路**：客户端可创建、更新、删除图表及导入 schema；写操作具有正确的 MCP tool annotations，并保留人工审批语义。
- **MCP-AC-04 一致性与安全**：更新冲突保留 409/revision 语义，401/403/404/409/422/5xx 映射稳定；服务不直连数据库、不记录 Token、不暴露任意 SQL。
- **MCP-AC-05 配置可用**：仅配置 `COLDRAWDB_BASE_URL` 和可选 `COLDRAWDB_ACCESS_TOKEN` 即可接入，四客户端文档包含可复制配置、启动、验证和故障排查步骤。

## 部署影响

- 是否需要部署：是
- 部署原因：新增可分发的 MCP stdio 可执行服务及客户端配置
- 影响环境：本地、测试、预发（不默认开放公网端口）
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（停止/移除 MCP 可执行文件和客户端配置，不影响现有 Web/API）
- 是否需要 smoke：是（本地 stdio MCP 初始化、工具发现和只读调用）

## UI/UX 变更声明

```yaml
ui_impact: false
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述（How）

先完成文档真值校正：统一原型入口、场景状态、资源索引和实现清单使用同一口径，并把本次审计的交互矩阵转为仓库内稳定回归。S06 严格按 Why → What → How 推进，先补需求和场景时序，再从时序图推导 MCP 工具输入、输出、错误和权限语义，最后形成测试与实现。

MCP 服务采用独立 Rust adapter/service，通过 `COLDRAWDB_BASE_URL` 调用既有 REST API；可选 `COLDRAWDB_ACCESS_TOKEN` 传递现有认证上下文。首批七个工具覆盖图表 CRUD 与 schema 导入/导出，读写属性、破坏性标记、revision 冲突和错误映射均在契约及编排测试中固定。四类客户端仅提供各自所需的配置外壳，共享同一 stdio 服务实现。
