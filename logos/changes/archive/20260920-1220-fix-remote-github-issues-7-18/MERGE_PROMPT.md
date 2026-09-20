# 合并指令

## 变更提案
- 提案名称：fix-remote-github-issues-7-18
- 提案目录：logos/changes/fix-remote-github-issues-7-18/

## 提案内容

# 变更提案：fix-remote-github-issues-7-18

> module: core | created: 2026-09-18
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues （OPEN #7～#18；#13 与 #14 内容完全重复，按一条处理，关闭 #13 保留 #14）

## 变更原因

远程仓库 `dorname/coldrawdb` 记录了 12 条 open issue（8 BUG + 1 UI + 2 Enhancement + 1 重复），需按 OpenLogos 变更流程修复并关闭。代码对照证据（已逐条核实当前工作树）：

| Issue | 类型 | 标题 | 根因摘要（对照当前工作树） |
|-------|------|------|---------------------------|
| #7 | BUG | 导入功能拖放区域无效 | `frontend-rs/src/editor_panels.rs:5661-5665` 的 `cdb-io-dropzone` 是纯展示 `<div>`，无 `dragover`/`drop` 事件、无隐藏 file input；textarea（`:5666-5676`）粘贴路径正常。 |
| #8 | BUG | 导入不支持 .ddl 文件 | 同上位置文案写死 `.sql / .dbml / .json`，无 accept 白名单；`.ddl` 仅在 `backend/src/phase3_bridge.rs:146` 等后端路径出现，前端导入 UI 未放行。 |
| #9 | BUG | 关系连线固定右出左进 | `frontend-rs/src/editor_render.rs:2896-2912` `calc_path` 写死 `x1=from.x+TABLE_WIDTH`（右出）、`x2=to.x`（左进）；`hit_test_field_port`（`:3867-3893`）虽能命中双侧 port，但 side 未被路径计算消费。 |
| #10 | Enhancement | 画布表/字段缺少中文注释展示 | 模型已有 `comment`（`editor_core.rs:70-101`、后端 entity 均有），但画布表头/字段行只画 `name`/`type_`（`editor_render.rs:3559-3606`）；Inspector 已有 comment 编辑入口（`editor_panels.rs:6498-6511`、`6619-6637`），仅画布未展示。 |
| #11 | BUG | 画布尺寸变化过程中元素失真 | canvas 缓冲尺寸与 DPR transform 仅在 `frame_tick` 驱动的渲染 effect 内同步（`editor_render.rs:986-1017`）；`splitter.rs:217-220` 拖动只写 CSS 变量不触发 `schedule_paint`；全仓库无 `ResizeObserver` / `window.resize` 监听，旧位图被拉伸直到下次交互。 |
| #12 | Enhancement | 关系连线、表头与表边框颜色配置 | `Table.color` 已存在且表头渐变已使用（`editor_render.rs:3542-3556`），但表边框仍用固定 `palette.table_border`（`:3535-3539`）；`Reference` 在前端模型（`editor_core.rs:114-124`）、后端 entity、`diagrams.yaml:291-303`、`mcp-tools.yaml` 均无 `color`；Inspector 无表/关系颜色入口。 |
| #13 / #14 | UI | 顶部「空间」返回按钮样式难看 | `editor_panels.rs:2722-2730` 返回按钮用 `cdb-btn--ghost`，但 `.cdb-btn--ghost`（`styles.css:3348-3357`）保留可见边框、`.cdb-btn` 无 `inline-flex` 居中；原型 `core-01-editor-prototype.html:361-362` 目标是透明边框 ghost（`.btn--ghost{border-color:transparent}`）。#13 与 #14 内容完全重复。 |
| #15 | BUG | 顶栏协作内容被截断 | `.cdb-room-badge`（`styles.css:832-847`）`max-width:180px` 且 `text-overflow:ellipsis` 写在容器 `<button>` 上，实际文本在内部 `<strong>`（无截断规则）；原型（`:82`）截断规则在 `strong` 上且 `max-width:190px`。AppBar 中部无弹性收缩策略。 |
| #16 | BUG | access token 失效太快（15 分钟） | `backend/src/auth/jwt.rs:6` 硬编码 `ACCESS_TTL_SECS = 900`；`backend/config.toml` 与 `init.rs:124-147` 配置结构体无 JWT TTL 项；env 覆盖仅支持 `COLDRAWDB_BIND_ADDR/DB_URL`（`init.rs:182-188`）；`auth.yaml:9/193/288-305` 描述写死 15m/900。refresh 链路（7/30 天）已完整。 |
| #17 | BUG | 拖动表时高亮边框为直角 | 静态选中框已用 `round_rect`（`editor_render.rs:3640-3655`），但 R-PERF-11 拖拽幽灵层（`:3477-3485`）用 CSS `outline` 模拟高亮——`outline` 不贴合 `border-radius:16px`，拖动时呈直角。 |
| #18 | BUG | MCP layout_diagram 返回格式校验失败 | `mcp-server/src/service.rs:320-363` `layout_diagram` 直接复用 `api.update` 返回并追加 `tables_repositioned`，未 normalize；`update_table/update_field/update_reference/update_diagram`（`:82-318`）同样直传上游响应；`protocol.rs:59-61` 同时返回 `content[].text`+`structuredContent`，宿主对 structuredContent 做严格校验时任何字段漂移即失败；`c5_canvas_tools.rs` 无 layout_diagram 契约测试。 |

## 变更类型

设计级变更（含接口级与代码级修复；#10/#12 增强需更新功能规格 / API / DB / 测试用例）

## 变更范围

- 影响的需求文档：无（存量能力缺陷修复与交互增强，不改产品 Why，不新增场景编号）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01d-import-export.md`（#7/#8 拖放交互 + `.ddl` 放行）
  - `prd/2-product-design/1-feature-specs/core-01b-relationship.md`（#9 按相对位置自动选择出入侧；#12 关系线颜色）
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（#11 尺寸变化 DPR 同步；#17 拖拽/选中圆角高亮；#10 注释展示）
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（#10 注释展示口径；#12 表头/边框颜色）
  - `prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`（#13/#14/#15 顶栏返回区与协作芯片）
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html`（#9/#10/#12 原型行为对齐；#13/#15 为生产反向对齐原型，原型仅微校）
- 影响的业务场景：S01（画布编辑/导入）、S03（Token 续期）、S06（MCP 工具契约）；无新场景编号
- 影响的部署方案：无（生产部署由后续 release 变更单独执行，遵循 `fix-remote-github-issues` 先例）
- 影响的 API：
  - `api/auth.yaml`（#16：TTL 可配置化描述 + expiresIn 示例更新）
  - `api/diagrams.yaml`（#12：Reference 增加 `color` 字段）
  - `api/mcp-tools.yaml`（#12：`UpdateReferenceInput` 增加 color；#18：澄清写工具成功响应为 outputSchema 瘦响应的约定）
- 影响的 DB 表：`reference` 新增 `color TEXT NOT NULL DEFAULT ''` 列（新迁移 `0006_reference_color`）
- 影响的编排测试：无（S03/S06 编排链路不变；TTL 默认值与返回体 normalize 不改端点契约）
- 影响的测试用例：`core-PC-import-export`、`core-PB-relationship`、`core-CR-canvas`、`core-RP-canvas-hidpi`、`core-PE-design-system`、`core-S03`、`core-S06` 测试用例 delta
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本提案交付本地可验证的规格 + 代码 + 测试；生产部署由后续 release 变更单独执行（遵循 `fix-remote-github-issues` 先例）
- 影响环境：本地
- 是否涉及数据迁移：是（`reference` 表 `ADD COLUMN color`，SQLite `ALTER TABLE` 向后兼容，默认 `''`，随应用启动迁移自动生效）
- 是否需要回滚预案：否（迁移提供配对 down.sql）
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: editor
    prototype: core-01-editor-prototype.html
    description: 画布连线自动选侧/注释展示/颜色配置行为对齐；顶栏返回区与协作芯片为生产反向对齐原型（原型仅微校）
```

## 变更概述

分六批交付同一提案，每批闭环（业务代码 + UT/ST + OpenLogos reporter，写入 `logos/resources/verify/test-results.jsonl`）：

**批次 A — 导入（#7、#8）**：为 `cdb-io-dropzone` 绑定 `dragenter/dragover/dragleave/drop`，读取 `DataTransfer.files` 文本写入导入内容并触发解析摘要；点击 dropzone 打开文件选择器兜底；扩展名白名单与文案放行 `.ddl`（按 SQL 路径解析）。

**批次 B — 画布渲染修复（#9、#11、#17）**：`calc_path` 按两表/字段相对几何位置自动选择出/入侧（近侧优先、少交叉），拖拽建关系时双侧 port 均可作起/终点，已有关系随拖动更新；新增 `ResizeObserver`（或 window resize + splitter 通知）驱动 canvas 缓冲尺寸与 DPR transform 同步，消除「需再聚焦才清晰」；拖拽幽灵层改用圆角描边（border 或 canvas 绘制）替代 CSS outline。

**批次 C — 顶栏 UI（#13/#14、#15）**：返回按钮对齐原型 ghost 语言（透明边框、inline-flex 居中、统一间距圆角）；room-badge 截断规则移至文本节点、预留最小可读宽度 + `title` 全文，中部区域建立弹性收缩策略，不与邀请按钮互相挤压。

**批次 D — 展示增强（#10、#12）**：画布表头/字段行在 `comment` 非空时展示中文注释（副标题/同行灰字 + hover tooltip 全文），提供显示开关（默认「英文名+注释」），空 comment 不留占位；表边框色跟随 `table.color`，Inspector 增加表颜色入口；`Reference` 增加 `color` 字段（前端模型 + 后端 entity + 迁移 0006 + `diagrams.yaml` + `mcp-tools.yaml`），关系线按 color 渲染、Inspector 增加关系颜色入口；JSON 导入导出保留颜色，SQL/DBML 降级忽略；未配置颜色时保持现有默认主题色。

**批次 E — auth TTL（#16）**：access token TTL 配置化（`config.toml` `[auth] access_ttl_secs` + `COLDRAWDB_ACCESS_TTL_SECS` 环境变量覆盖），默认值由 900s 提升为 **3600s（1 小时，用户已拍板）**；`/auth/login` 与 `/auth/refresh` 返回的 `expiresIn` 与实际 JWT `exp` 保持一致；`auth.yaml` 同步。

**批次 F — MCP 契约（#18）**：`layout_diagram` 与写工具（`update_table/update_field/update_reference/update_diagram`）返回前 normalize 为与各自 `outputSchema` 完全一致的瘦响应；补充 mock 端到端契约测试（structuredContent 必须通过 outputSchema 校验）；`mcp-tools.yaml` 澄清返回约定。

## 用户确认结论（2026-09-18）

1. **#13/#14 重复**：已确认——关闭 #13（标记 duplicate），集中到 #14 跟踪。
2. **#16 默认 TTL**：已拍板默认 **3600s（1 小时）**，`config.toml [auth] access_ttl_secs` 与 `COLDRAWDB_ACCESS_TTL_SECS` 可覆盖。
3. **#10 默认显示模式**：已确认默认「英文名+注释」，可切换「仅英文名 / 仅注释」；空 comment 行为与现状一致。
4. **#12 颜色持久化范围**：颜色仅在 JSON 导入导出与 DB 中保留，SQL/DBML 导出降级忽略（不改变 DDL 语义）。


## 需要合并的 Delta 文件

### 1. deltas/api/auth.yaml

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/api/auth.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/database/coldrawdb-v1.sql

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/database/coldrawdb-v1.sql`
- 目标目录：`logos/resources/database/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 11. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 12. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-PB-relationship-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 13. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 14. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-PE-design-system-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 15. deltas/test/core-RP-canvas-hidpi-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-RP-canvas-hidpi-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 16. deltas/test/core-S03-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-S03-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 17. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-remote-github-issues-7-18/deltas/test/core-S06-test-cases.md`
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
   git add -A && git commit -m "docs(fix-remote-github-issues-7-18): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-remote-github-issues-7-18`。
