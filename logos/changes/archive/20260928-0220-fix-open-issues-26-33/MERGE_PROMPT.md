# 合并指令

## 变更提案
- 提案名称：fix-open-issues-26-33
- 提案目录：logos/changes/fix-open-issues-26-33/

## 提案内容

# 变更提案：fix-open-issues-26-33

> module: core | created: 2026-09-27
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues （OPEN #26～#33）

## 变更原因

远程仓库当前 8 条 open issue（4 BUG + 2 UX + 2 Enhancement）需按 OpenLogos 变更流程修复并关闭。逐条根因摘要（代码对照证据来自 issue 正文与当前工作树核实）：

| Issue | 类型 | 标题 | 根因摘要 |
|-------|------|------|----------|
| #26 | BUG | MCP 大图写入单次载荷过大 | `update_diagram` 全量写回无体积上限约定、无官方分批策略、超限错误不可诊断；细粒度工具（`update_table` 等）存在但缺少分批指引与 revision 串行规则文档 |
| #27 | BUG | 区域画完后无法调整大小 | 区域仅 `area_rect_from_drag` 创建一次定尺寸 + `area_drag` 整框平移；无边/角 resize 命中与落账路径；Inspector `inspector-area-form` 仅 name/color/删除，无宽高字段（表侧已有 `feat-table-resize` 可对照） |
| #28 | Enhancement | 统一修改所有关系线条类型 | `reference.line_type`（bezier/orthogonal/straight）数据模型与单条 Inspector 下拉已存在（`fix-open-issues-19-22` 交付），缺口仅在 bulk apply / 全局入口与一次 Undo |
| #29 | BUG | SQL 导入有关系无注释、数据库导入有注释无关系 | 后端 `bridge_introspect` IR 已收集 `IntrospectedTable.fks`，但 `tables_to_json`/`connect_response_data` 未序列化 references；前端 `parse_bridge_import_tables` 只包 `{"tables": ...}`；UT-PC-26 甚至断言「import/connect 不产 references」。SQL 侧 COMMENT 解析需按失败样例加固（schema 限定名、dump 方言） |
| #30 | UX | 缩小画布字体不清晰 | 无 LOD/分档简化渲染；小缩放下文字发糊、关系线过细，无法辨认拓扑（`editor_render` 已有 `R-PERF-07` zoom 分档基础） |
| #31 | UX | 选中表高亮偏弱 | 规格 `core-01b-relationship.md` 已要求「非相关线 opacity ≤ 0.25」但实现偏弱/未充分落地；选中表描边与相关连线强调不足 |
| #32 | BUG | 注释在渐变表头上对比度差 | 表头注释压在渐变亮/暗段上可读性差；字段行注释浅灰细字；无前景色自适应或衬底策略（`CommentDisplay` / `draw_table` 表头渐变） |
| #33 | Enhancement | 统一并美化图表视觉风格与语义图标 | 表卡/PK 标记/关系端点/ToolRail 图标视觉语言不统一；需更新设计 token（`core-07`）、图标库（`core-08`）与主原型后对齐前端渲染 |

## 变更类型

设计级变更（含接口级与代码级修复：#29 涉及 bridge API 响应结构；#26 涉及 MCP 工具合同；#27/#28/#30/#31/#32/#33 涉及功能规格 + 原型 + 前端渲染）

## 变更范围

- 影响的需求文档：
  - `prd/1-product-requirements/core-S06-mcp-service-requirements.md`（#26 分批写入验收条件）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（#27 区域 resize 交互合同；#30 小缩放 LOD 渲染分档）
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（#32 注释可读性/对比度合同）
  - `prd/2-product-design/1-feature-specs/core-01b-relationship.md`（#28 批量线型入口；#31 选中高亮落地口径）
  - `prd/2-product-design/1-feature-specs/core-01d-import-export.md`（#29 两路径对称验收口径）
  - `prd/2-product-design/1-feature-specs/core-03-bridge-io.md`（#29 import/connect 输出 references）
  - `prd/2-product-design/1-feature-specs/core-07-design-tokens.md`（#32 注释对比度 token；#33 视觉体系统一）
  - `prd/2-product-design/1-feature-specs/core-08-icon-library.md`（#33 语义图标统一）
  - `prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`（#26 分批写入约定与错误可诊断）
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html`（#27/#28/#30/#31/#32/#33 原型对齐，主原型为唯一现行入口）
- 影响的业务场景：S01（画布编辑/导入）、S06（MCP）；无新场景编号
- 影响的 API：
  - `api/bridge.yaml`（#29：import/connect 响应增加 references）
  - `api/mcp-tools.yaml`（#26：分批写入约定、体积上限与超限错误合同）
  - `api/diagrams.yaml`（#28：如需批量更新端点则新增，否则仅语义澄清——优先复用既有整体保存路径，避免新端点）
- 影响的 DB 表：无（#28 复用既有 `reference.line_type` 列；#29 为响应结构变化，无 schema 变更）
- 影响的编排测试：无
- 影响的测试用例：
  - `core-PC-import-export-test-cases.md`（#29 对称用例；废止「import/connect 不产 references」断言）
  - `core-CR-canvas-test-cases.md`（#27 区域 resize；#30 LOD）
  - `core-PB-relationship-test-cases.md`（#28 批量线型；#31 选中高亮）
  - `core-PE-design-system-test-cases.md`（#32 对比度；#33 视觉体系）
  - `core-S06-test-cases.md`（#26 分批写入）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本提案交付本地可验证的规格 + 代码 + 测试；生产部署由后续 release 变更单独执行（遵循 `fix-open-issues-19-22` 先例）
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: editor
    prototype: core-01-editor-prototype.html
    description: "#27 区域 resize 手柄与 Inspector 宽高；#28 批量线型入口；#30 小缩放 LOD；#31 选中高亮增强；#32 注释对比度；#33 视觉体系与语义图标统一"
```

## 变更概述

分批交付同一提案，每批闭环（业务代码 + UT/ST + OpenLogos reporter，写入 `logos/resources/verify/test-results.jsonl`）：

**批次 A — #29 导入对称**：后端 `import/connect` 响应序列化 references（由 introspect fks 展开），前端 `parse_bridge_import_tables` 合并进导入 store；SQL 路径按失败样例加固 COMMENT 解析（schema 限定名、常见 dump 方言）；同一 PG fixture 两条路径均出线+注释的对称用例；废止 UT-PC-26 反向断言。

**批次 B — #27 区域 resize**：选中区域显示边/角 resize 手柄，拖拽改宽高并走与区域拖动一致的落账/撤销路径；Inspector 增加宽高（或 x/y/w/h）编辑；只读模式禁用；最小尺寸约束复用创建拖框阈值口径。

**批次 C — #28 批量线型**：提供「应用到全部关系」入口（Inspector 选中关系时 + 命令面板命令），批量修改走一次 CommandStack 事务保证单次 Undo；范围默认全部关系；只读禁用；保存后刷新保持。

**批次 D — #31 + #32 可读性渲染**：落地规格已有口径——选中表时相关连线加粗/提亮（≥1.5×）、非相关表/线降不透明度（≤0.25）；注释对比度：表头注释前景色按表头色自适应（或半透明衬底），提高字号/字重，字段注释提亮；亮/暗主题成对验证。

**批次 E — #30 小缩放 LOD**：zoom 低于阈值时隐藏字段行、仅保留表名（+注释一行）并加粗关系线；复用 `R-PERF-07` zoom 分档机制；阈值与样式入设计 token。

**批次 F — #26 MCP 分批写入**：工具合同与文档明确 `update_diagram` 建议体积上限与分批指引（update_table/update_field/update_reference/layout 串行 expected_revision）；超限返回可诊断错误（当前体积、建议分片粒度）；评估并优先采用「合同 + 错误可诊断 + 文档约定」MVP，partial patch / batch_update 工具作为可选增强在同批内按复杂度裁量。

**批次 G — #33 视觉体系统一**：先更新主原型与设计 token（表卡、PK/FK 等语义图标、关系端点、ToolRail 图标），再对齐前端 canvas 绘制与图标组件；亮/暗主题成对设计。

## 产品拍板（按 issue 建议默认锁定，若需改口请在确认前指出）

1. **#26**：MVP 采用「官方分批约定 + 超限可诊断错误」，不引入 partial update API；若实现中发现细粒度工具有缺口再补 batch 工具。
2. **#27**：画布手柄 + Inspector 宽高双入口都做；最小尺寸复用创建阈值。
3. **#28**：入口 = Inspector「应用到全部关系」+ 命令面板；一次批量 = 一次 Undo；本期不联动 stroke_style。
4. **#30**：采用 LOD 分档（低 zoom 隐藏字段行 + 加粗线），不做位图超分等重手段。
5. **#31**：按规格口径落地：相关线加粗提亮、非相关表/线降透明度。
6. **#32**：表头注释前景色按表头色自适应对比度，必要时加半透明衬底；不引入新主题。
7. **#33**：以更新主原型 + 设计 token 为先导，再改前端渲染；不照搬外部工具风格。
8. **#29**：references 由后端在 import/connect 响应直接输出，前端合并；SQL 注释解析按对称 fixture 加固。


## 需要合并的 Delta 文件

### 1. deltas/api/bridge.yaml

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/api/bridge.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 11. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 12. deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 13. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 14. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 15. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/test/core-PB-relationship-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 16. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 17. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/test/core-PE-design-system-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 18. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-26-33/deltas/test/core-S06-test-cases.md`
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
   git add -A && git commit -m "docs(fix-open-issues-26-33): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-open-issues-26-33`。
