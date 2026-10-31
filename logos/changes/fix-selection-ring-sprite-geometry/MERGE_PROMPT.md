# 合并指令

## 变更提案
- 提案名称：fix-selection-ring-sprite-geometry
- 提案目录：logos/changes/fix-selection-ring-sprite-geometry/

## 提案内容

# 变更提案：修复缩放后选中环与卡体精灵几何不一致

> module: core | created: 2026-10-31 | issue: #52

## 变更原因

Issue #52 报告表卡在缩放和字号倍率变化后出现浅蓝选中环与卡体边缘错位、底部空白死区。当前 `draw_table_selection` 每帧按精确 `zoom / label_font_scale / comment_mode / tier` 计算选中环，而卡体精灵指纹仅混入 2px 粗粒度 `effective_font_bucket`。同一 zoom 分档和字体桶内，两个 zoom 可产生不同表头高、字段行高和卡体总高，却复用旧精灵，导致活画选中环绕过旧卡体。

## 变更类型

设计级变更（精灵缓存指纹合同与回归测试；不改变产品交互和视觉设计）。

## 变更范围

- 影响的需求文档：无新增需求；落实 Issue #52 的既有正确性验收。
- 影响的功能规格：`core-01-editor-canvas.md` §5 R-PERF-07。
- 影响的业务场景：S01 画布缩放、字号切换、注释模式与维度切换。
- 影响的测试：`core-CR-canvas-test-cases.md` 新增 UT/ST-CR-SPRITE-02。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的编排测试：无。
- 影响的原型：无；主原型不包含生产 Canvas 精灵缓存实现。
- 影响的部署与 smoke：无。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos Canvas 缓存键修复，无服务配置、数据库或部署流程变化。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

将当前有效表卡宽高的精确几何指纹混入精灵缓存键。即使表内容、zoom 分档和粗字体桶相同，只要有效宽高随 zoom、字号、注释模式或 LOD 档位变化，就必须重光栅；坐标和选中态仍不进指纹，保持拖动/平移位块传输优化。

新增纯函数回归，证明“同粗字体桶、不同几何”必须产生不同缓存指纹；新增浏览器几何探针回归，直接比较实际卡体精灵与选中环的宽高，覆盖双向缩放、150% 字号、注释模式和 Table/Field 维度。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-selection-ring-sprite-geometry/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-selection-ring-sprite-geometry/deltas/test/core-CR-canvas-test-cases.md`
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
   git add -A && git commit -m "docs(fix-selection-ring-sprite-geometry): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-selection-ring-sprite-geometry`。
