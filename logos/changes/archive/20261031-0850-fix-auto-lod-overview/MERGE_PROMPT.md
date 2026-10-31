# 合并指令

## 变更提案
- 提案名称：fix-auto-lod-overview
- 提案目录：logos/changes/fix-auto-lod-overview/

## 提案内容

# 变更提案：恢复全景自动 LOD 并保留显式维度覆盖

> module: core | created: 2026-10-31 | issue: #59

## 变更原因

Issue #59 指出 16 表 / 19 关系模型缩放到 zoom≈0.1 后仍保持字段详情档，表卡字段全部展开并互相遮挡。现行规格和实现把 `ViewDimension` 限定为 Table/Field 双态且默认 Field，明确禁止 zoom 自动切档，因此手动画「表维度」虽可解决，默认全景体验仍不成立。

本次通过 Delta 修订既有显式双态合同：新增 Auto 默认模式，低缩放自动进入拓扑；用户显式选择 Table/Field 后继续锁定维度，不受 zoom 影响。

## 变更类型

设计级变更（画布维度状态、LOD 判档和既有相反测试预期修订）。

## 变更范围

- 影响的功能规格：`core-01-editor-canvas.md` §5.12；`core-07-design-tokens.md` LOD 阈值登记。
- 影响的业务场景：S01 画布缩放、全景浏览和关系选择。
- 影响的 API：无。
- 影响的 DB 表：无；localStorage 仍使用 `cdb.view-dimension`，新增 `auto` 值。
- 影响的测试：`core-CR-canvas-test-cases.md` UT/ST-CR-LOD-01，新增 UT/ST-CR-LOD-02；低缩放字段测试需显式锁定 Field。
- 主原型无维度切换实现，本次无原型变更。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos 前端视图状态和测试，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否；旧 `table`/`field` 偏好原样有效，缺省/非法值回落 Auto。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

`ViewDimension` 改为 `Auto / Table / Field` 三态，默认 Auto，切换顺序为 Auto → Table → Field → Auto。Auto 在 `zoom <= 0.5` 使用 Topology，高于阈值使用 Detail；Table/Field 继续固定 Topology/Detail。状态指示显示“维度：自动”，三入口和持久化共用同一状态。

渲染、关系命中、字段/表命中、悬停摘要统一使用带 zoom 的判档函数，避免显示与点击几何不一致。新增自动边界纯函数测试和 16 表全景浏览器回归，并修订旧“任何 zoom 都不切档”的测试为仅对显式模式成立。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-auto-lod-overview/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/fix-auto-lod-overview/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-auto-lod-overview/deltas/test/core-CR-canvas-test-cases.md`
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
   git add -A && git commit -m "docs(fix-auto-lod-overview): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-auto-lod-overview`。
