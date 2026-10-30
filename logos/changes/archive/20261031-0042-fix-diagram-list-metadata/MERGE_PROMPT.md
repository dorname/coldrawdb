# 合并指令

## 变更提案
- 提案名称：fix-diagram-list-metadata
- 提案目录：logos/changes/fix-diagram-list-metadata/

## 提案内容

# 变更提案：修复图表列表摘要元数据

> module: core | created: 2026-10-31 | issue: #53

## 变更原因

Issue #53 指出 MCP `list_diagrams` 及其上游 `GET /diagrams/queryAll` 的摘要始终返回 `revision=0`、`updated_at=null`，而 `get_diagram` 中同一图已有真实 revision 和修改时间，导致同名图无法按新旧区分。

根因是遗留列表查询仍使用旧 SeaORM `diagram::Model` / `DiagramVo` 映射，该模型只含 `last_modified`，没有读取迁移后新增的 `revision`、`updated_at` 列；MCP 规范化收到缺字段后按既有兼容逻辑回退为 `0/null`。

## 变更类型

接口级变更（补齐既有摘要字段的真实值，不新增端点，不改变 MCP 工具名称或参数）。

## 变更范围

- 影响的需求文档：`core-S06-mcp-service-requirements.md` FR-S06-02。
- 影响的功能规格：`core-S06-mcp-service-design.md` §4.1。
- 影响的业务场景：S04 创建房间关联图候选、S06 图表发现。
- 影响的 API：`GET /diagrams/queryAll` 响应项、MCP `list_diagrams` 的 `DiagramSummary` 契约。
- 影响的 DB 表：`diagram`（只读取既有 `revision`、`updated_at`，不改 Schema）。
- 影响的测试：`core-S04-test-cases.md`、`core-S06-test-cases.md`。
- 不影响：正式 `/api/v1/diagrams/{id}` 单图结构、列表过滤/排序/limit、软删除过滤、数据库迁移、可选 table/reference 计数。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地后端查询映射与 MCP 摘要回归，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

遗留 `queryAll` 列表查询直接读取 `diagram` 的既有元数据列，响应项同时保留原有 `lastModified` 兼容字段，并返回真实 `revision` 与 `updated_at`；`updated_at` 缺失时仅为旧数据回退到 `lastModified`，不得用常量或当前时间伪造。

MCP `list_diagrams` 保持现有过滤、名称排序和 limit 行为，只透传真实元数据。新增后端真实 SQLite 回归与 MCP 规范化回归，确保 revision=8 等值不会在列表链路中退化为 0/null。


## 需要合并的 Delta 文件

### 1. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/fix-diagram-list-metadata/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md

- Delta 文件：`logos/changes/fix-diagram-list-metadata/deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md

- Delta 文件：`logos/changes/fix-diagram-list-metadata/deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/fix-diagram-list-metadata/deltas/test/core-S04-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-diagram-list-metadata/deltas/test/core-S06-test-cases.md`
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
   git add -A && git commit -m "docs(fix-diagram-list-metadata): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-diagram-list-metadata`。
