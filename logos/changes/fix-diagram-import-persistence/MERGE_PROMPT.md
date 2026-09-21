# 合并指令

## 变更提案
- 提案名称：fix-diagram-import-persistence
- 提案目录：logos/changes/fix-diagram-import-persistence/

## 提案内容

# 变更提案：修复导入图持久化缺陷导致的 MCP 响应校验失败

## 变更原因

用户通过 MCP 客户端（Claude Code）调用 coldrawdb 工具时，**每次对导入图的调用都出现 MCP 响应校验错误**（`Output validation error: Invalid structured content`），`layout_diagram` 等写工具表现异常。实证定位的完整因果链：

1. `POST /api/v1/diagrams/import` 处理中，`diagram_from_import_payload` 用 `serde_json::from_value::<Vec<TableDto>>` 反序列化 payload 表；`TableDto.id` / `FieldDto.id` 为必填且无默认值，**payload 中表/字段缺 `id` 时整个数组反序列化失败**，被 `.ok()` 静默吞掉 → 存入空表。
2. 同函数把 `payload.name` 的 `None` 传入 `save_diagram`，后者 `UPDATE diagram SET name=NULL` **覆盖了**导入 handler 第一步 INSERT 的默认名 `imported_diagram` → 产生 `name = NULL` 的脏图。
3. `import_diagram_v1` 对 `persist_import_payload` 的结果 `let _ =` 丢弃，且 `imported_tables` / `imported_fields` 按 payload 原始数量上报 → 客户端看到「导入 1 表成功」，实际落库 0 表。
4. MCP 契约 `Diagram.name` 声明为 `{type: string}`（必填）。此后对该图的任何 `get_diagram`（以及所有内部先 GET 的写工具）返回 `name: null`，违反契约 → 客户端每次调用都抛响应校验错误。

## 变更类型

接口级变更（import 端点响应语义 + MCP 契约 schema 修正）+ 代码修复 + 存量数据迁移。

## 变更范围

- 影响 API：
  - `logos/resources/api/diagrams.yaml` — `POST /api/v1/diagrams/import` 响应语义：`imported_tables` / `imported_fields` 按**实际持久化数量**上报；持久化失败返回 5xx 而非 200 虚报；`warnings` 细化丢弃明细
  - `logos/resources/api/mcp-tools.yaml` — `Diagram.name` 类型放宽为 `[string, "null"]`（防御存量与绕过迁移的脏数据，杜绝 MCP 校验硬失败）
- 影响的业务场景：S06（MCP 工具链）
- 影响的编排测试：`logos/resources/test/core-S06-test-cases.md` — 新增导入回归用例（缺 id payload 导入后 GET 校验、name 兜底）
- 影响的 smoke 测试：`logos/resources/test/smoke/core-smoke-test-cases.md` — 新增导入→GET 冒烟
- 影响的代码：
  - `backend/src/diagrams_v1.rs` — `import_diagram_v1`：传播持久化错误、实报数量、warnings 明细
  - `backend/src/diagram_persistence.rs` — `diagram_from_import_payload`：逐表容错（跳过坏表而非整组丢弃）+ 缺 id 自动生成 + name 缺省与 handler 一致；`save_diagram`：`name = None` 时不覆盖已有列值
  - `mcp-server/src/service.rs` — `import_schema`：payload 前置规范化（补 `name` 默认值、为缺 id 的表/字段生成 `auto-` id），使修复对未升级后端同样生效
- 影响的 DB 表：`diagram`（存量迁移：`name IS NULL` → `'imported_diagram'`）

## 部署影响

- 是否需要部署：是
- 部署原因：后端修复与数据迁移需重新部署后端服务方可生效（本地为 docker compose 重建镜像）
- 影响环境：本地 / 测试
- 是否涉及数据迁移：是（刷存量 `name IS NULL` 的图，防止既有脏图继续触发 MCP 校验失败）
- 是否需要回滚预案：否（迁移为幂等 UPDATE，旧版本后端可继续运行）
- 是否需要 smoke：是（部署后导入→GET 冒烟验证持久化与 name 兜底）

## 变更概述

本次变更修复导入链路「假成功、真丢数据」的持久化缺陷，并消除其引发的 MCP 响应校验硬失败。后端侧：导入反序列化改为逐表容错并为缺失 id 自动生成；`name` 缺省贯穿全链且 `save_diagram` 不再用 NULL 覆盖；持久化结果不再被吞，数量按实报。MCP adapter 侧：`import_schema` 对 payload 做同样的前置规范化，保证只升级 adapter 的旧后端也能正确落库。契约侧：`Diagram.name` 放宽 nullable 防御存量脏数据；import 响应语义与实现拉齐。配套 S06 回归用例与部署后冒烟，并附存量数据迁移。


## 需要合并的 Delta 文件

### 1. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/fix-diagram-import-persistence/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/fix-diagram-import-persistence/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-diagram-import-persistence/deltas/test/core-S06-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/fix-diagram-import-persistence/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
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
   git add -A && git commit -m "docs(fix-diagram-import-persistence): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-diagram-import-persistence`。
