# 变更提案：fix-relation-mouse-hit-precision

> module: core | created: 2026-09-30

## 变更原因

用户目标反馈：关系线通过鼠标无法精确选中。既有 fix-issues-42-44 / #44 与 fix-issue-47 / #47 已解决线型同源、最近距离优先、8 屏幕像素阈值、zoom 感知等核心命中问题；但绘制侧关系线端点记号（crow's foot / single bar，外接尺寸 `REL_ENDPOINT_SIZE=10px`）独立于主线存在，命中检测 `dist_to_reference` 仅计算点到线距离，未覆盖端点记号几何。当鼠标落在端点符号外延（距端点中心 ≤10px 但距主线 >8px）时，视觉上已“点在线条末端”，实际上却不命中，造成“无法精确选中”的体感。

## 变更类型

设计级变更

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
  - §4.7 关系线命中合同（新增 R-HIT-06 端点记号命中热区）
- 影响的业务场景：S01 编辑器画布 / 关系线选中
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-PB-relationship-test-cases.md`
  - 新增 UT-PB-24：端点记号外侧点命中关系
  - 新增 ST-PB-14：点击端点记号附近 10px 热区选中关系

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端命中逻辑调整，不涉 API、DB、配置或数据迁移；本地 dev / 已部署 staging 前端镜像重新 build 后即可生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

在 V1 边界内采用最小可行方案：

1. **端点记号纳入关系线命中热区**：`dist_to_reference` 对 `bezier` / `orthogonal` / `straight` 三种线型，在既有主线距离基础上，额外计算指针到两端端点中心 `(x1,y1)` / `(x2,y2)` 的距离，热区半径取 `REL_ENDPOINT_SIZE`（10px，与绘制端点记号外接尺寸同源）。单条关系最终距离 = `min(主线距离, 端点距离)`，再参与 `d * zoom <= 8.0` 的 8 屏幕像素阈值判定。
2. **保持最近距离优先与优先级不变**：端点热区仅作为单条关系的额外命中面，多线端点重叠处仍按 R-HIT-02 返回距离最小者；关系命中整体仍排在表命中之后（R-HIT-05）。
3. **规格与测试同步**：在 `core-01b-relationship.md` §4.7 新增 R-HIT-06；在 `core-PB-relationship-test-cases.md` 新增 UT-PB-24 / ST-PB-14，覆盖 bezier/orthogonal/straight 与 zoom=1.0 端点外侧命中。