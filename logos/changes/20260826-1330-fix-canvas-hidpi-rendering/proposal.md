# 变更提案：fix-canvas-hidpi-rendering

> module: core | created: 2026-08-26
> 前置：core 模块已 launched（2026-08-20 smoke 通过 / 2026-08-24 verify 通过）

## 变更原因

用户反馈"页面渲染结果模糊尤其是字体，需要提供清晰度"。经排查，根因集中于 Canvas 渲染层：

1. **DPR 缩放缺失**：`editor_render.rs:323-328` 设置 canvas backing store 像素时直接用 `parent.client_width()`，**未乘以 `window.devicePixelRatio`**。在 Retina / HiDPI 屏（dpr ≥ 2）上，CSS 像素1920 对应画布像素只有1920×1，浏览器把低分辨率画布拉伸到屏幕上，所有线条、字段文字、表名、PK 标记、type 标签均模糊。
2. **Canvas 字体降级**：`CANVAS_FONT = "\"Plus Jakarta Sans\", sans-serif"`，但规格要求 Plus Jakarta Sans 走 Google Fonts 异步加载。画布 `fill_text` **不受 CSS font-display 控制**，若 web font 加载未完成即触发渲染，会直接降到 `sans-serif`，9~13px 的小字模糊。
3. **Canvas 抗锯齿不可控**：CSS 端设置了 `-webkit-font-smoothing: antialiased`，但 Canvas 2D `fill_text` 使用浏览器默认文字栅格化策略，画布文字与 DOM 文字抗锯齿不一致。
4. **缩放下 DPR 不同步**：`ctx.scale(t.zoom, t.zoom)` 仅作用于变换，未与 devicePixelRatio 联合，导致用户缩放画布后再次丢失清晰度。
5. **字号密度偏高**：表头 13px、字段名 11px、type 10px、PK 标记 9px，在 DPR=1 下边缘锐利度已经紧张，DPR=2 不处理则雪上加霜。

> 主题不属于本变更范围（`core-0b-dark-mode` 已落地，仅 dark/light 模式切换），
> 动效不属于本变更范围（`core-0c-motion` 已落地，动画期间清晰度另有规则）。

## 变更类型

实现级变更（修改 `frontend-rs/src/editor_render.rs`、`index.html`、`frontend-rs/src/styles.css`，
更新 `core-01-editor-canvas.md`、`core-07-design-tokens.md`、`core-0c-motion.md` 与测试用例）。

## 变更范围

- 影响的文档（规格层）：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` 新增 §10 HiDPI 渲染基线
  - `logos/resources/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` §10 字体段补充 web-font 等待契约
  - `logos/resources/prd/2-product-design/1-feature-specs/core-0c-motion.md` §3 增加"渲染稳定期"过渡约束
- 影响的代码：
  - `frontend-rs/src/editor_render.rs` §4（resize 钩子）— DPR 缩放
  - `frontend-rs/src/editor_render.rs` §draw_canvas — `set_image_smoothing_enabled(true)`、DPR 缩放复位
  - `frontend-rs/index.html` — `font-display: optional`（替代 `swap`，避免降级闪烁）
  - `frontend-rs/src/styles.css` — `text-rendering: optimizeLegibility`、`image-rendering: auto` 基线
- 影响的测试用例：新增 `core-RP-canvas-hidpi-test-cases.md`（UT-RP-*、ST-RP-*）
- 影响的编排测试：无
- 影响的 smoke：HP-01~HP-05 画布截图回归
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无

## 部署影响

- 是否需要部署：否（V1 已上线，本变更为前端渲染优化，无需重启后端）
- 部署原因：无
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（Trunk 构建产物替换 `frontend-rs/dist/`，保留上一版 dist）
- 是否需要 smoke：是（playwright 截图前后对比基线 `frontend-rs/test-results/baseline-canvas.png`）

## 变更概述

### 交付物

1. **DPR 自适应画布**
   - 监听 `window.devicePixelRatio` 变化（`matchMedia('(resolution: ...)').addEventListener('change', ...)`）
   - canvas backing store 像素 = CSS 像素 × DPR
   - `ctx.scale(dpr × zoom, dpr × zoom)` 复位后再绘制
   - 调整 viewport 时（含 sidebar 折叠、浏览器缩放）一并重新计算

2. **画布字体等待契约**
   - `document.fonts.ready` 完成后才允许首帧 `draw_canvas` 提交（避免空白期 + 错位）
   - `set_image_smoothing_enabled(true)`（线条锐利）
   - 字号表显式分 DPR 渲染（dpr=1 保持 13/11/10/9，dpr≥1.5 上浮 1px）

3. **CSS 全局抗锯齿基线**
   - `text-rendering: optimizeLegibility`（DOM 文字 + 连字）
   - `image-rendering: auto`（图标 / PNG）
   - `-webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale`（已在，确认保留）

4. **Playwright 视觉回归**
   - 在 1×、1.5×、2× 三个 DPR 下截取画布，按 95% 像素相似度作为 PASS 阈值

### 不在本变更范围

- 关系连线 / 区域框 / 便签的字号调整（属于 UX 微调，单独提案）
- 字号 token 全局上浮（涉及大量组件级验收，单独提案）
- Monaco 编辑器等宽字体（属于 Monaco 挂载子任务，`core-0a-code-editor.md`）

## 风险

| 风险 | 缓解 |
|---|---|
| 已有截图基线（HP-01~HP-05）因 DPR 改变而 diff | smoke 新增 `canvas-pixel-density` 步骤，记录 DPR=1/2 baseline |
| `document.fonts.ready` 在弱网下长时间不 resolve | 兜底超时（3s）后强制首帧 |
| 缩放下 `ctx.scale` 累乘 | 每次 `draw_canvas` 开头 `setTransform(dpr*zoom, 0, 0, dpr*zoom, 0, 0)` 复位 |