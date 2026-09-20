# 合并指令

## 变更提案
- 提案名称：fix-open-issues-19-22
- 提案目录：logos/changes/fix-open-issues-19-22/

## 提案内容

# 变更提案：fix-open-issues-19-22

> module: core | created: 2026-09-20
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues （OPEN #19～#22）

## 变更原因

远程仓库当前 4 条 open issue（2 BUG + 2 Enhancement）需按 OpenLogos 变更流程修复并关闭。代码对照证据（已逐条核实当前工作树）：

| Issue | 类型 | 标题 | 根因摘要（对照当前工作树） |
|-------|------|------|---------------------------|
| #19 | BUG | 区域拖动松手误打开邀请模态 | `editor_render.rs` `on_pointerup` 开头提前 `release_pointer_capture`（约 L2272–2274）；有效拖动后浏览器合成的 `click` 按松手坐标命中 AppBar `btn-invite`（`editor_panels.rs` L2913），打开 `modal-invite`。典型 pointer capture 松手 click 穿透。 |
| #20 | BUG | 表名/字段名过长时画布表宽不自适应 | 默认硬编码 `TABLE_WIDTH = 230.0`（`editor_render.rs` L19）；`compute_table_render_size` 仅消费 `table.width.unwrap_or(TABLE_WIDTH)`；注释展示（R-CMT-01/02）为单行省略、不扩宽；Set Table Width「0 = auto」实际写入 `Some(0)`，无 `measure_text` → 写回宽度路径。 |
| #21 | Enhancement | 导入后关系连线过密过花 | #9 已修自动选侧；路径仍以三次贝塞尔为主（`calc_path` / `draw_bezier_fields`）；无线条类型/线型数据模型；无密度降噪；规格 `core-01b-relationship.md` §3.4 明确 V1.2 不做正交折线/绕障——本 issue 为后续增强入口。 |
| #22 | Enhancement | 表颜色改为选择器 + 出边跟随表色 | Inspector 表色仅为 `TABLE_COLOR_PRESETS` 命名下拉（`editor_panels.rs` L6311 / L6716）；`relation_stroke_color`（`editor_render.rs` L3359）只看 `Reference.color`，空则主题色，**不**继承源表 `Table.color`。 |

## 变更类型

设计级变更（含代码级修复；#20/#21/#22 需更新功能规格 / 测试用例；#21 线型若落库则含 API/DB；#19 主要为交互合同 + 代码）

## 变更范围

- 影响的需求文档：无（存量能力缺陷修复与交互增强，不改产品 Why，不新增场景编号）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（#19 拖动后 click 抑制；#20 表宽自适应合同）
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（#20 内容自适应宽度；#22 颜色选择器 + 出边继承约定交叉引用）
  - `prd/2-product-design/1-feature-specs/core-01b-relationship.md`（#21 线条类型/线型 + 密度降噪；#22 空 color 继承源表色）
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html`（#21/#22 原型微对齐：关系线型入口、表色 picker）
- 影响的业务场景：S01（画布编辑/导入）；无新场景编号
- 影响的部署方案：无（生产部署由后续 release 变更单独执行）
- 影响的 API：
  - `api/diagrams.yaml`（#21：`Reference` 增加 `line_type` / `stroke_style` 可选字段；#22 语义澄清：空 `color` = 跟随源表）
- 影响的 DB 表：`reference` 新增 `line_type` / `stroke_style` 列（新迁移，默认兼容现有贝塞尔+实线）
- 影响的编排测试：无
- 影响的测试用例：`core-CR-canvas`、`core-PB-relationship`、可选 e2e（#19 拖动不误开邀请）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本提案交付本地可验证的规格 + 代码 + 测试；生产部署由后续 release 变更单独执行（遵循 `fix-remote-github-issues-7-18` 先例）
- 影响环境：本地
- 是否涉及数据迁移：是（`reference` 表 `ADD COLUMN line_type` / `stroke_style`，SQLite 默认值向后兼容）
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
    description: "#19 拖动穿透防护；#20 表宽按内容自适应；#21 关系线型/虚线与密度降噪；#22 表色 picker + 出边跟随源表色"
```

## 变更概述

分四批交付同一提案，每批闭环（业务代码 + UT/ST + OpenLogos reporter，写入 `logos/resources/verify/test-results.jsonl`）：

**批次 A — #19 拖动 click 穿透**：有效拖动（超过 `DRAG_THRESHOLD`）结束后抑制紧随其后的 document/`btn-invite` `click`；`pointerup` 落账完成后再释放 capture（或交给浏览器自动释放，避免松手坐标命中顶栏）；同类防护覆盖表/便签/区域拖动。e2e：区域 `pointerdown` → 移到 `btn-invite` → `pointerup`，断言 `modal-invite` 不出现。

**批次 B — #20 表宽内容自适应**：新增按当前注释显示模式测量表名/字段名/类型/注释的最小渲染宽，夹在 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`（建议 max=480，对齐 ListView 列宽上限）；`width == None` 或 `Some(0)`（「0 = auto」）走自适应；显式正数宽度仍尊重用户手动设置。导入/新建表后可一次性 fit；渲染与 hit-test 共用 `compute_table_render_size`。

**批次 C — #21 线型 + 密度降噪（MVP）**：
- C1 样式：`Reference.line_type` ∈ {`bezier`（默认）/ `orthogonal` / `straight`}；`stroke_style` ∈ {`solid`（默认）/ `dashed`}；Inspector/关系浮层可切换并落库；正交折线消费既有 `pick_port_sides`，不回归 #9。
- C2 降噪：选中表/关系时，非相关连线降低不透明度；相关连线保持/高亮。
- **本变更不做**：导入后自动整理布局 / 边捆绑 / 绕障 routing（issue 建议批次 B）——关闭 #21 时开 follow-up 跟踪布局整理。

**批次 D — #22 表色 picker + 出边跟随**：Inspector 表色在预设快捷色之外增加 `<input type="color">` / 自定义 hex；`Reference.color` 为空时渲染用色 = 源表（start 端）`Table.color`，源表亦空则主题默认色；关系显式设色优先，清除关系色后恢复跟随。

## 产品拍板（按 issue 建议默认锁定，若需改口请在确认前指出）

1. **#20**：采用内容自适应（measure → 夹紧），`0`/`None` = auto；手动正数宽度优先。
2. **#21**：本变更交付样式（C1）+ 密度降噪（C2）；自动布局整理另开 follow-up。
3. **#22**：空关系色跟随源表；显式关系色覆盖；预设 + native color picker。


## 需要合并的 Delta 文件

### 1. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/database/coldrawdb-v1.sql

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/database/coldrawdb-v1.sql`
- 目标目录：`logos/resources/database/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-open-issues-19-22/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-open-issues-19-22): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-open-issues-19-22`。
