# 合并指令

## 变更提案
- 提案名称：mcp-canvas-tools
- 提案目录：logos/changes/mcp-canvas-tools/

## 提案内容

# 变更提案：mcp-canvas-tools

> module: core | created: 2026-09-18
> 关联场景：S06（AI 客户端通过 MCP 管理数据库图表）

## 变更原因

用户（AI 客户端使用者）反馈：MCP 现有 7 个 tools 仅覆盖 diagram 级 CRUD + import/export，缺少**画布内部编辑能力**——无法通过 AI 客户端完成字段关系连线、表/字段名与注释修改、画布布局美化等操作。这迫使开发者必须在 Web UI 中手动完成这些编辑，违背了「AI 客户端全链路管理」的设计初衷。

## 变更类型

需求级变更（新增 4 个 MCP tools → 影响 S06 场景时序、API 契约、MCP adapter 代码、UT/ST 测试；后端零改动——复用既有 GET/PUT 全量保存语义）

## 变更范围

- 影响的需求文档：无（S06 既有需求不变）
- 影响的功能规格：无（不改变前端行为）
- 影响的业务场景：`prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md` — 新增画布编辑时序与工具推导
- 影响的部署方案：无
- 影响的 API：`api/mcp-tools.yaml` — 新增 4 个 tool 定义（`update_table` / `update_field` / `update_reference` / `layout_diagram`）+ 相应 schemas
- 影响的 DB 表：无
- 影响的编排测试：无（不新增场景编号）
- 影响的 smoke 测试：无
- 影响的 UT/ST 测试：`test/core-S06-test-cases.md` — 新增 UT-MCP-16～22（7 个 UT）、ST-MCP-10～12（3 个 ST）
- 影响的代码：`mcp-server/src/service.rs`（4 个 tool handler）、`mcp-server/src/api.rs`（如需要）、`mcp-server/src/layout.rs`（新文件：力导向布局算法）、`mcp-server/tests/`（对应测试代码）

## 部署影响

- 是否需要部署：否（MCP 服务随用户本机启动，无服务器端部署）
- 部署原因：N/A
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **新增 4 个细粒度 MCP tools**（全部通过既有 `GET /api/v1/diagrams/{id}` + `PUT /api/v1/diagrams/{id}` 完成，零后端改动）：
   - `update_table`：修改表名 / 表注释 / 位置 / 颜色 / 锁定状态
   - `update_field`：修改字段名 / 注释 / 类型 / 约束（not_null / primary / default）
   - `update_reference`：创建 / 更新 / 删除字段间关系连线（含基数、级联约束）
   - `layout_diagram`：力导向自动布局（Force-directed），按 reference 关联聚类排列表
2. **布局算法**：MCP adapter 内置纯函数力导向布局（Fruchterman-Reingold 变体），确定性输出（固定随机种子），无副作用。
3. **所有新 tools 遵守既有安全约束**：`expected_revision` 乐观锁、`update_diagram` 全量写回语义、409 冲突处理与既有约定一致。
4. 新增 UT-MCP-16～22（参数校验 / 布局算法正确性 / 工具契约稳定性），ST-MCP-10～12（端到端画布编辑链路）。


## 需要合并的 Delta 文件

### 1. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/mcp-canvas-tools/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md

- Delta 文件：`logos/changes/mcp-canvas-tools/deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/mcp-canvas-tools/deltas/test/core-S06-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(mcp-canvas-tools): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive mcp-canvas-tools`。
