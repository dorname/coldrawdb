# 合并指令

## 变更提案
- 提案名称：fix-issue-49-name-comment-relation-anchor
- 提案目录：logos/changes/fix-issue-49-name-comment-relation-anchor/

## 提案内容

# 变更提案：fix-issue-49-name-comment-relation-anchor

> module: core | created: 2026-09-29

## 变更原因

GitHub issue #49：当画布注释显示模式切换为「仅英文名」时，关系线端点与表卡边缘之间出现明显空隙/断线。根本原因是关系线锚点与选侧计算仍按写死的 `CommentDisplay::NameComment` 估算表宽，而表体绘制已按当前 `comment_mode` 变窄，导致端点落在当前卡边之外。

本提案修复锚点/选侧与绘制口径不一致的问题，确保任意注释模式下关系线均贴齐当前模式下的表卡边缘。

## 变更类型

代码级修复（含规格澄清与测试用例补充）。

## 变更范围

- **影响的需求文档**：无
- **影响的功能规格**：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §5.10 表卡内容自适应宽度
  - `core-01-editor-canvas.md` §5.16 画布标签字号 / 注释显示模式
  - `core-01-editor-canvas.md` §5.12 小缩放 LOD 渲染分档（表维度锚点）
- **影响的业务场景**：S01 编辑器画布（core-S01）
- **影响的部署方案**：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` 部署注记
- **影响的 API**：无
- **影响的 DB 表**：无 schema 变更
- **影响的编排测试**：无新增场景文件
- **影响的单元/场景测试**：`logos/resources/test/core-CR-canvas-test-cases.md`
- **影响的 smoke 测试**：`logos/resources/test/smoke/core-smoke-test-cases.md` 中 SMOKE-core-STABLE-01 全量回归即可，不新增用例

## 部署影响

- **是否需要部署**：是
- **部署原因**：纯前端画布渲染逻辑变更，前端产物随镜像；无 DB migration、无 API 变更。
- **影响环境**：staging / 生产
- **是否涉及数据迁移**：否
- **是否需要回滚预案**：是（回退上一镜像即可）
- **是否需要 smoke**：是（既有套件全量回归）

## 变更概述

1. **锚点/选侧与绘制同 comment_mode 口径**：
   - `field_anchor_for_side_with`、`anchor_for_tier_with` 不再写死 `CommentDisplay::NameComment`，改为接收并使用调用方传入的 `comment_mode`。
   - `pick_port_sides` 同步传入当前 `comment_mode`，确保关系线起止侧判断基于当前模式下的卡宽。
   - `calc_path_tier_with` / `calc_orthogonal_path_tier_with` / `calc_straight_path_tier_with` 调用链透传 `comment_mode`。

2. **命中测试同口径**：
   - `hit_test_with`、`hit_test_field_with`、`hit_test_field_port_with`、`hit_test_endpoint_with` 及 pointer 事件处理统一使用当前 `comment_mode` 计算 effective width/height。

3. **规格与测试**：
   - 在 `core-01-editor-canvas.md` 明确 R-WIDTH-08 / R-LOD-08 的「同口径」包含 `comment_mode` 维度。
   - 新增 `UT-CR-ANCHOR-01`：三种注释模式下字段锚点 x 坐标等于 `table.x` / `table.x + resolve_table_width_for(table, mode, ...)`。
   - 新增 `ST-CR-ANCHOR-01`：e2e 在「仅英文名」/「英文名+注释」/「仅注释」三模式间切换，断言关系线始终贴齐当前模式卡边。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-issue-49-name-comment-relation-anchor/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-issue-49-name-comment-relation-anchor/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-issue-49-name-comment-relation-anchor/deltas/test/core-CR-canvas-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issue-49-name-comment-relation-anchor): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issue-49-name-comment-relation-anchor`。
