# Delta 索引与合并顺序

> proposal: align-unified-prototype-and-add-mcp | module: core | 2026-08-18

## 1. 合并顺序

1. `reference/project/logos-project.yaml`：场景计数、S03～S05 状态、S06 与资源索引，由 merge 执行者按指令应用。
2. `prd/1-product-requirements/`：先合并场景总览，再新增 S06 Why。
3. `prd/2-product-design/`：校正唯一主原型与生产状态，再合并 S06 What。
4. `prd/3-technical-plan/2-scenario-implementation/`：先合并 S06 时序，再处理工具/API 契约。
5. `api/diagrams.yaml`：以实际 Rust handler 为准全量替换漂移的 v1 OpenAPI。
6. `api/mcp-tools.yaml`：新增七工具契约。
7. `test/` 与 `scenario/`：新增/修改测试定义和 MCP 编排。
8. `reference/implementation/`、架构、部署及根文档状态：最后按参考指令更新统计和实现清单。

## 2. 关键决策

- 唯一现行原型是 `core-01-editor-prototype.html`；三个独立协作原型只保留历史参考。
- S03～S05 真实状态是“后端完成、生产前端待接入”。
- S06 MVP 是本地 stdio，不含远程 transport。
- list 临时走遗留 `/diagrams/queryAll`；其他 CRUD/import 走 `/api/v1/diagrams*`。
- export 在 adapter 内做纯函数序列化。
- diagram API 当前未强制 JWT；不得在合并文档中声称已经具备 diagram 级授权。

## 3. Delta 文件统计

| 类别 | 文件 |
|---|---:|
| 项目索引/根文档 | 3 |
| 需求/设计/架构/场景/部署 | 10 |
| API/MCP 契约 | 2 |
| 测试/编排/实现清单 | 4 |
| Mermaid 源与 SVG | 2 |

> SVG 是 `.mmd` 的验证导出，不作为可编辑源；后续修改必须先改 `.mmd`、重新验证，再覆盖 SVG。
