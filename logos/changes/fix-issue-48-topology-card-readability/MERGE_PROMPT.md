# 合并指令

## 变更提案
- 提案名称：fix-issue-48-topology-card-readability
- 提案目录：logos/changes/fix-issue-48-topology-card-readability/

## 提案内容

# 变更提案：fix-issue-48-topology-card-readability

> module: core | created: 2026-09-29

## 变更原因

issue #48 反馈：切换到表维度（Topology / ViewDimension::Table）后，表卡变成窄条胶囊，英文表名与中文注释并排展示时经常被 `…` 截断，全局浏览关系拓扑时难以辨认表意。根本原因是表维度使用了与字段维度相同的 `TABLE_WIDTH_MAX = 480` 上限，且表维度下字号经 LOD 补偿后变大，相同字符占用更多像素，导致中英文并排长文本更容易被截断。

## 变更类型

设计级变更

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
  - §4.6 LOD / R-LOD-02（表维度表头渲染与截断口径）
  - §4.6 R-WIDTH-06/07/08（effective 字号与宽度自适应合同）
- 影响的业务场景：S01 编辑器画布（核心画布渲染）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的测试用例：`logos/resources/test/core-CR-canvas-test-cases.md`
  - 新增 UT-CR-TOPO-WIDTH-01：表维度宽度上限高于字段维度
  - 新增 ST-CR-TOPO-WIDTH-01：表维度下长中文注释表卡完整可读

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端渲染参数调整，不涉 API、DB、配置或数据迁移；本地 dev / 已部署 staging 前端镜像重新 build 后即可生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

采用最小可行方案：**为表维度单独放宽卡宽上限**。新增常量 `TABLE_WIDTH_MAX_TOPOLOGY = 640.0`（高于字段维度 `TABLE_WIDTH_MAX = 480.0`），在 `resolve_table_width_for` 中根据 `LodTier::Topology` 使用更高上限。`lod_table_size_with`、`anchor_for_tier_with`、`field_anchor_for_side_with` 等已通过 `resolve_table_width_for` 同源取宽，因此关系锚点会自动与新宽度对齐（#49 已确保 comment_mode 一致性）。

规格侧同步修订 R-LOD-02：表维度仍按单行表名+注释渲染，但超长截断阈值放宽到 `TABLE_WIDTH_MAX_TOPOLOGY`；同时强调表维度字号补偿不得导致文本被卡宽截断（与 R-WIDTH-06 一致）。新增 UT 验证上限差异，新增 ST 验证典型中英文表卡在表维度下完整可读。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-issue-48-topology-card-readability/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-issue-48-topology-card-readability/deltas/test/core-CR-canvas-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issue-48-topology-card-readability): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issue-48-topology-card-readability`。
