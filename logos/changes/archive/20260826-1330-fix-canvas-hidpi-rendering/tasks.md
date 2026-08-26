# 任务清单 — fix-canvas-hidpi-rendering

> created: 2026-08-26 | status: pending

## Task 1 — 规格 delta
- [ ] `core-01-editor-canvas.md` 新增 §10 HiDPI 渲染基线（DPR 缩放、字号策略、画布文字抗锯齿）
- [ ] `core-07-design-tokens.md` §10 字体段补充 web-font ready 契约、`font-display: optional`、字号 DPR 系数表
- [ ] `core-0c-motion.md` §3 增加"渲染稳定期"过渡约束（避免 web font 闪烁期动画叠加）
- [ ] 新增 `core-RP-canvas-hidpi-test-cases.md`（UT-RP-01~05、ST-RP-01~03）

## Task 2 — 实现批次 A：DPR 自适应画布
- [ ] `editor_render.rs::setup_canvas_resize_observer` 新增 DPR 感知
- [ ] `set_transform(dpr*zoom, 0, 0, dpr*zoom, 0, 0)` 替代 `ctx.scale(t.zoom, t.zoom)`
- [ ] matchMedia DPR 变化监听 + 触发重绘
- [ ] 关闭 `image_smoothing_enabled = false` 避免线条被双线性插值

## Task 3 — 实现批次 B：字体等待契约
- [ ] `lib.rs` 监听 `document.fonts.ready` + 3s 兜底超时
- [ ] 首帧仅在 font-ready 后入队 `request_animation_frame`
- [ ] `CANVAS_FONT` 改 `document.fonts.check` 探测后的实际可用字体，否则降级 ui-monospace
- [ ] `index.html` Google Fonts URL 加 `&display=optional` 参数

## Task 4 — 实现批次 C：CSS 全局基线
- [ ] `styles.css :root` 增加 `text-rendering: optimizeLegibility`
- [ ] `styles.css :root` 增加 `image-rendering: auto`
- [ ] 复核所有 `--cdb-font-size-*` 应用处，确认 selector 继承

## Task 5 — 测试与回归
- [ ] UT-RP-01：dpr=1 画布 backing store 像素 = CSS 像素
- [ ] UT-RP-02：dpr=2 画布 backing store 像素 = 2× CSS 像素
- [ ] UT-RP-03：缩放从 100% → 150% → 200% 不重复累乘
- [ ] UT-RP-04：matchMedia DPR 变化触发 redraw
- [ ] UT-RP-05：font-ready 超时（3s）后仍可首帧
- [ ] ST-RP-01：playwright 在 1× / 2× 截图 95% 像素相似度
- [ ] ST-RP-02：playwright 模拟 web font 加载失败 → 不卡首帧
- [ ] ST-RP-03：HP-01~HP-05 smoke 仍 PASS

## Task 6 — 文档与归档
- [ ] 更新 `core-implementation-checklist.md` §2.4（编辑器渲染）勾选 DPR 行
- [ ] 跑 `openlogos verify`，输出 `VERIFY_PASS` 到本目录
- [ ] 跑 `openlogos smoke`，输出 `SMOKE_PASS` 到 `logos/resources/verify/smoke-report.md`
- [ ] 走 `/openlogos:merge fix-canvas-hidpi-rendering` → `/openlogos:archive fix-canvas-hidpi-rendering`