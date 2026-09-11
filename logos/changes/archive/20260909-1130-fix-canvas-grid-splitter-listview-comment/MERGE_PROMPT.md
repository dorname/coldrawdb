# 合并指令

## 变更提案
- 提案名称：fix-canvas-grid-splitter-listview-comment
- 提案目录：logos/changes/fix-canvas-grid-splitter-listview-comment/

## 提案内容

# 变更提案：画布网格横线 + 分隔条拖动方向 + 列表视图 comment 核查

> module: core | created: 2026-09-09

## 变更原因

用户实机反馈三个问题（2026-09-09）：

1. **画布多出大量横线**（截图证据）：`fix-canvas-zoom-invite-comment-resize` 引入的 R-PERF-02 网格点阵合批绘制中，`GridSink::dot` 在同一 path 内连续调用 `arc()` 而未先 `move_to`。按 Canvas2D 规范，`arc` 会从当前路径点到弧线起点补一条直线——于是每行网格点被连成水平线、行末到下一行行首连成对角线，形成截图中的横条纹与对角阴影。
2. **分隔条拖动方向与宽度调整方向相反**（实机复现：向右拖 120px，`--cdb-inspector-w` 从 330px 涨到 450px）：`drag_width` 对两个实例统一用 `start_w + dx`，但 `splitter-inspector` 位于右侧面板左缘，向右拖应变窄（`-dx`）；`splitter-list-tree` 位于左侧树列右缘，`+dx` 正确。core-09 §14 规格未写方向语义，是实现期漏判。
3. **列表视图 comment 展示 + 数据库导入注释采集**：经当前构建实机全链路验证，该需求**已实现且工作正常**，本次不改代码，仅记录核查结论（见下）。

## 问题3 核查结论（无需改动，附实机证据）

- 列表视图表名 comment：左侧表树节点在表名下方渲染注释行（`list-table-comment-*`，core-01a §1.4.1 / UT-PC-29）。
- 列表视图字段名 comment：字段网格「显示名称」列即 comment（`list-field-comment-*` 受控草稿 + blur 落账）。
- DDL 导入：PG `COMMENT ON TABLE/COLUMN` 与 MySQL 内联 `COMMENT 'x'` 均解析回填（UT-PC-07）。
- 数据库连接导入：后端 PG introspect 经 `obj_description`/`col_description` 采集注释（SQLite 无 COMMENT 语法属引擎限制；MySQL 连接导入未支持）。
- 实机证据（2026-09-09，localhost:8080 当前构建 + 独立 PG 17.5）：DDL 导入 `COMMENT ON` → 表树显示「订单主表」、字段列显示「主键编号」；PG 连接导入 `postgres://…:55433` → 表树显示「订单主表PG」、字段列显示「订单号」。截图 `/tmp/db-import-listview.png`。

## 变更类型

代码级修复

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-09-core-components.md` §14.2（补充分隔条方向语义，堵规格漏洞）
- 影响的业务场景：无（不改变场景行为定义）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的代码：
  - `frontend-rs/src/editor_render.rs` — `GridSink::dot` 补 `move_to`（问题1）
  - `frontend-rs/src/splitter.rs` — `drag_width` 引入按实例方向的符号（问题2），同步更新 `mod tests` 中锁定旧行为的断言

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端渲染/交互修复，本地 trunk 构建即生效，无服务端或环境变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **问题1**：`editor_render.rs` 的 `CtxGridSink::dot` 在 `arc` 前加 `move_to(x + r, y)`，切断同 path 内弧间连线；UT-CR-BATCH-01 的 mock sink 调用计数口径保持不变。
2. **问题2**：`splitter.rs` 为 `SplitterKind` 增加方向语义——Inspector（右侧面板左缘）拖动位移取反（`start_w - dx`），ListTree（左侧树列右缘）保持 `start_w + dx`；键盘方向键维持数值语义（ArrowLeft 减宽 / ArrowRight 加宽，与 `aria-valuenow` 一致）不变。更新 `mod tests` 中 Inspector 方向断言。
3. **规格堵漏**：core-09 §14.2 增补一条方向语义条款（右侧面板左缘分隔条：左拖变宽、右拖变窄；左侧列右缘分隔条：右拖变宽、左拖变窄）。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md

- Delta 文件：`logos/changes/fix-canvas-grid-splitter-listview-comment/deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
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
   git add -A && git commit -m "docs(fix-canvas-grid-splitter-listview-comment): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-canvas-grid-splitter-listview-comment`。
