# 变更提案：perf-canvas-relation-hit-index

> module: core | created: 2026-09-30

## 变更原因

用户目标：“画布性能优化，让所有操作更丝滑”。

当前 `hit_test_reference_tier` 在 pointermove / hover / 点击命中时遍历全部关系，对每条关系调用 `dist_to_reference` 并计算 bezier / orthogonal / straight 的精确距离。当关系数量达到产品愿景中“100 张表 / 200 条关系”规模时，每次命中检测都成为主线程上的 O(n_refs) 瓶颈，导致拖拽、缩放后的悬浮/选中等操作出现可感知的迟滞。

本变更在保持 R-HIT-01~06 语义与“最近距离优先”不变的前提下，引入关系线 AABB 预过滤：先用廉价的外扩包围盒剔除远离指针的关系，再对剩余候选执行精确距离计算，从而在大多数交互位置把均摊开销降到 O(k)，k 为热区附近的关系数。

## 变更类型

设计级变更（新增 R-HIT-07 命中性能合同 + 对应纯函数与测试）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md` §4.7 新增 R-HIT-07 关系线命中 AABB 预过滤
- 影响的业务场景：S01 编辑器画布 / 关系线选中与悬浮
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：
  - `logos/resources/test/core-PB-relationship-test-cases.md` 新增 UT-PB-25 / UT-PB-26
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端命中逻辑性能优化，不涉 API、DB、配置或数据迁移；前端重新 build 后生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **AABB 预过滤**：在 `frontend-rs/src/editor_render.rs` 新增纯函数 `reference_aabb(tables, r) -> Rect`，根据 `line_type` 计算关系线的轴对齐包围盒（覆盖 bezier / orthogonal / straight 三种线型的全部几何控制点）。`hit_test_reference_tier` 在调用 `dist_to_reference` 前，将包围盒按命中阈值外扩（max(8px 主线带宽, 10px 端点热区) 按当前 zoom 换算为世界距离）；若指针世界坐标落在外扩 AABB 之外，直接跳过该关系。
2. **结果等价性不变**：任何可能命中（主线距离 ≤ 8 屏幕像素或端点距离 ≤ 10 屏幕像素）的点，必然落在外扩 AABB 内；因此预过滤不会导致漏命中。多线竞争场景仍按 R-HIT-02 取距离最小者。
3. **规格与测试同步**：在 `core-01b-relationship.md` §4.7 新增 R-HIT-07；在 `core-PB-relationship-test-cases.md` 新增 UT-PB-25（200 条关系远距离点击返回 None 且 `dist_to_reference` 不被调用）与 UT-PB-26（200 条关系近距离点击仍返回正确 ref_id）。
