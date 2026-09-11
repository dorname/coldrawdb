# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5.6 新增 R-PERF-07（表精灵缓存）/ R-PERF-08（DOM 写守护）/ R-PERF-09（endpoint_drag 空写守卫）/ R-PERF-10（拖拽降采样渲染）/ R-PERF-11（表拖拽幽灵层）
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 登记 UT-CR-SPRITE-01 / UT-CR-GUARD-01 / UT-CR-DPR-01 / UT-CR-GHOST-01 / ST-CR-GHOST-01，并修订 ST-CR-PAN-01 平移断言口径（≤ 事件数+1，R-PERF-10 松手恢复帧）

## [code] 代码实现
- [x] `editor_render.rs`：表精灵离屏缓存（draw_table 拆卡体/选中环，指纹 + zoom 分档 + 主题/dpr 失效，drawImage 位块传输）
- [x] `editor_render.rs`：渲染 effect 中 follow_path / rubber_d / data-follow-path 变更才写
- [x] `editor_render.rs`：endpoint_drag 最近字段未变时不写 store
- [x] `editor_render.rs`：R-PERF-10 拖拽降采样（capped_drag_dpr + RENDER_DPR 有效 dpr 贯通 backing store/CTM/精灵指纹）
- [x] `editor_render.rs`：R-PERF-11 表拖拽幽灵层（无关系线表 pointermove 只改 CSS transform 主画布零重绘；有关系线/transform 变化回退逐帧路径；退出路径 Drop 兜底撤除）
- [x] 新增 UT-CR-SPRITE-01 / UT-CR-GUARD-01 / UT-CR-DPR-01 / UT-CR-GHOST-01（host 纯函数），保持 UT-CR-BATCH-01 等既有断言通过
- [x] 前端 `cargo test` 全绿（273 passed）+ 实机复测拖拽窗口帧间隔 p99=16.8ms ≤ 17ms（基线 33.4ms，3 轮稳定）+ F 批 canvas-perf e2e 全过（ST-CR-PAN-01 / ST-CR-INSP-01 / ST-CR-GHOST-01）
