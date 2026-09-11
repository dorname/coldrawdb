# 变更提案：画布拖动流畅度优化（表精灵缓存 + DOM 写守护）

> module: core | created: 2026-09-09

## 变更原因

用户反馈：画布操作（移动元素等）不够顺畅丝滑，要求继续优化流畅度。

实机测量基线（2026-09-09，Playwright + dpr=2 + 6 表）：

- 拖动中 fps≈60 但 **p95/p99 帧间隔 33.4ms**（每 ~20 帧掉 1 帧），是「不丝滑」的直接体感来源。
- Canvas JS 侧调用耗时合计仅 ~1ms/帧（arc 38 万次/65.8ms 为调用计数，非光栅成本），说明瓶颈在**每帧全量重光栅**与**每帧 DOM 写**：
  1. 每张表每帧重画 `shadowBlur=16` 投影 + 每帧新建 `create_linear_gradient` + 每字段行 `fill_text`×3 + `set_font` 字符串拼接——dpr=2 下光栅成本 ×4。
  2. 渲染 effect 每帧无条件 `follow_path.set(...)` + `canvas.set_attribute("data-follow-path")` + `rubber_d.set(None)`——值未变也触发 Leptos SVG 更新与 DOM 属性写。
  3. 关系端点拖动（endpoint_drag）分支每个 pointermove 都 `store.references.set(...)` 整 Vec 落账——即使最近字段未变化。

## 变更类型

代码级修复（性能优化）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`core-01-editor-canvas.md` §5.6（新增 R-PERF-07 / R-PERF-08 / R-PERF-09 / R-PERF-10 / R-PERF-11）
- 影响的业务场景：无（不改变交互语义，仅降低每帧成本）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试文档：`core-CR-canvas-test-cases.md`（新增 UT-CR-SPRITE-01 / UT-CR-GUARD-01 / UT-CR-DPR-01 / UT-CR-GHOST-01）
- 影响的代码：
  - `frontend-rs/src/editor_render.rs` — 表精灵离屏缓存、渲染 effect DOM 写守护、endpoint_drag 空写守卫、拖拽降采样渲染、表拖拽幽灵层

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端渲染优化，本地 trunk 构建即生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **R-PERF-07 表卡片精灵缓存**：`draw_table` 拆为「卡体（含投影/渐变/文字，不含选中环）」与「选中环（保留每帧活画）」。卡体按 `(表内容指纹, 主题, dpr, zoom 分档)` 缓存到离屏 canvas，拖动/平移期间每帧仅 `drawImage` 位块传输；指纹变更（编辑落账）才重光栅。zoom 分档 `k ∈ {1, 2}`（zoom≤1 取 1，否则取 2），sprite 比例 `s = dpr × k`，zoom>2 时允许轻微软化以换取缓存稳定（手势缩放跨档才重建）。
2. **R-PERF-08 渲染 effect DOM 写守护**：`follow_path` / `rubber_d` 信号与 `data-follow-path` 属性仅在值变化时写入（与上一帧字符串比较），消除每帧 Leptos SVG 更新与属性序列化。
3. **R-PERF-09 endpoint_drag 空写守卫**：最近字段 id 未变化时不调 `store.references.set`，避免拖动中每 move 整 Vec 落账 + 订阅者通知。
4. **R-PERF-10 拖拽降采样渲染**：07/08/09 落地后复测发现主线程绘制 p99 仅 2.2ms，掉帧根因实为 SwiftShader 软件合成——主画布内容每帧变化时，合成器按「损伤区域 = 整个画布元素」重光栅输出（dpr=2 下 ~19ms/帧）。拖拽活跃期把有效 dpr 压到 1（像素数 ÷4），松手下一帧恢复全分辨率；绘制路径（backing store / CTM / 精灵指纹）统一经 `effective_device_pixel_ratio()` 读取。
5. **R-PERF-11 表拖拽幽灵层**：R-PERF-10 复测确认合成成本与画布 backing 分辨率弱相关（输出损伤区域不变，~15ms/帧残留），要消除拖拽中帧必须让主画布在拖拽期间零重绘——无关系线的被拖表改由绝对定位的小 canvas（幽灵层，`data-testid="drag-ghost"`）经 CSS transform 移动，每帧损伤区域缩到表卡大小；有关系线的表（线条需跟随）与拖拽中 zoom/pan 变化回退 R-PERF-10 逐帧路径；pointerup/pointercancel/Escape 任一退出路径自动撤除幽灵层（Drop 语义兜底）。

验收口径：实机同口径复测（dpr=2 + 6 表拖动），帧间隔 p99 从 33.4ms 降至 ≤17ms；既有 F 批 canvas-perf e2e 与全部 UT/ST 保持通过。
