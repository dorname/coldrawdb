# Delta — 项目根文档状态对齐

> proposal: align-unified-prototype-and-add-mcp | merge targets: README.md / AGENTS.md / CLAUDE.md

## MODIFIED — 项目状态统一口径

三个根文档的“项目当前状态/能力/统计”必须使用以下事实，不得各自维护冲突数字：

- 现行 HTML 主原型：1 个，`core-01-editor-prototype.html`；三个 S03/S04/S05 独立原型是历史参考。
- S01/S02：生产前后端已实现。
- S03/S04/S05：后端 auth/rooms/collab REST、DB、WS 与测试已实现；生产前端登录、房间、WS/OT/presence 尚未接入。
- 现有生产后端路由：diagram v1 5 + bridge 5 + auth 5 + rooms 11 + collab REST 2 + WS 1；遗留 `/diagrams/*` 路由单列，不混入 v1 统计。
- 数据表：V1 11 张 + V2 auth/rooms/collab 增量表；具体数量以 migrations/DDL 与架构文档为准，不再沿用“11 张即全量”的旧口径。
- S06：MCP stdio 服务规划中，目标客户端 Claude、Codex、Cursor、OpenCode；MVP 七工具，不包含 Streamable HTTP。

## ADDED — MCP 快速入口（README）

merge 后 README 增加“MCP（规划/实现中）”小节，链接 S06 需求、设计、工具契约与测试，不在实现完成前写“已支持”。代码实现并 verify 通过后，才把状态切换为“已支持”。

## MODIFIED — 方法论状态（AGENTS/CLAUDE）

将场景数更新为 S01～S06，并明确活跃变更 `align-unified-prototype-and-add-mcp`。保留所有 OpenLogos guard、人类确认点和中文输出规则，不得以本 delta 覆盖用户已有方法论约束。

## MODIFIED — 场景规格顶部元数据（人工合并指令）

Markdown Delta 只能锚定正式章节，以下位于首个章节之前的元数据由 merge 执行者精确替换：

- `core-S03-user-auth-design.md`：原型改为 `core-01-editor-prototype.html`；追加历史参考 `core-03-auth-prototype.html` 和“后端已实现、生产前端待接入”。
- `core-S04-room-lifecycle-design.md`：原型改为 `core-01-editor-prototype.html`；追加历史参考 `core-04-collab-prototype.html` 和“后端已实现、生产前端待接入”。
- `core-S05-ot-collab-design.md`：原型改为 `core-01-editor-prototype.html`；追加历史参考 `core-05-ot-collab-prototype.html` 和“后端已实现、生产前端待接入”。

该指令只修改三份文件的顶部引用，不改变章节内容或语义边界。
