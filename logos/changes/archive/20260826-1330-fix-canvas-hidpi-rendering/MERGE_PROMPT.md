# 合并指令

## 变更提案
- 提案名称：fix-canvas-hidpi-rendering
- 提案目录：logos/changes/archive/20260826-1330-fix-canvas-hidpi-rendering/

## 提案内容

# 变更提案：fix-canvas-hidpi-rendering

> module: core | created: 2026-08-26
> 前置：core 模块已 launched（2026-08-20 smoke 通过 / 2026-08-24 verify 通过）

## 变更原因

用户反馈"页面渲染结果模糊尤其是字体，需要提供清晰度"。经排查，根因集中于 Canvas 渲染层：

1. **DPR 缩放缺失**：`editor_render.rs:323-328` 设置 canvas backing store 像素时直接用 `parent.client_width()`，**未乘以 `window.devicePixelRatio`**。在 Retina / HiDPI 屏（dpr ≥ 2）上，CSS 像素1920 对应画布像素只有1920×1，浏览器把低分辨率画布拉伸到屏幕上，所有，线条、字段文字、表名、PK 标记、type 标签均模糊。
2. **Canvas 字体降级**：`CANVAS_FONT = "\"Plus Jakarta Sans\", sans-serif"`，但规格要求 Plus Jakarta Sans 走 Google Fonts 异步加载。画布 `fill_text` **不受 CSS font-display 控制**，若 web font 加载未完成即触发渲染，会直接降到 `sans-serif`，9~13px 的小字模糊。
3. **Canvas 抗锯齿不可控**：CSS 端设置了 `-webkit-font-smoothing: antialiased`，但 Canvas 2D `fill_text` 使用浏览器默认文字栅格化策略，画布文字与 DOM 文字抗锯齿不一致。
4. **缩放下 DPR 不同步**：`ctx.scale(t.zoom, t.zoom)` 仅作用于变换，未与 devicePixelRatio 联合，导致用户缩放画布后再次丢失清晰度。
5. **字号密度偏高**：表头 13px、字段名 11px、type 10px、PK 标记 9px，在 DPR=1 下边缘锐利度已经紧张，DPR=2 不处理则雪上加霜。

## 变更范围

- 影响的文档（规格层）：
  - `core-01-editor-canvas.md` 新增 §11 HiDPI 渲染基线
  - `core-07-design-tokens.md` §10 字体段补充 web-font 等待契约
  - `core-0c-motion.md` §2.1 渲染稳定期过渡约束
- 影响的代码：
  - `frontend-rs/src/editor_render.rs`：DPR 感知 backing store、set_transform 复位、matchMedia 监听、字体探测
  - `frontend-rs/src/lib.rs`：`wait_for_fonts_with_fallback` 3000ms 兜底
  - `frontend-rs/index.html`：`display=swap` → `display=optional`
  - `frontend-rs/src/styles.css`：`text-rendering: optimizeLegibility`、`image-rendering: auto`
  - `frontend-rs/Cargo.toml`：`web-sys` features 新增 FontFace / FontFaceSet / MediaQueryList
  - `frontend-rs/Trunk.toml`：wasm_opt = 0 绕开 binaryen bulk-memory 校验缺失
  - `logos/logos.config.json`：smoke sandbox_root `/private/tmp` → `/tmp`
  - `scripts/common.sh` / `scripts/start-local.sh`：新增 `COLDRAWDB_FRONTEND_TIMEOUT=60s`
  - `scripts/smoke-local-scripts.sh`：覆盖 SMOKE-core-01~05 本地 E2E

## 合并的提交

```
e536ab9 feat(fix-canvas-hidpi-rendering): DPR-aware canvas backing store + web-font ready contract
4920e86 fix(fix-canvas-hidpi-rendering): 修编译错误 + wasm-opt bulk-memory 兼容
f4f2e38 fix(fix-canvas-hidpi-rendering): smoke sandbox_root /private/tmp → /tmp
af16cce fix(smoke): COLDRAWDB_FRONTEND_TIMEOUT=60s，trunk serve 冷启动不再超时
266ee51 feat(smoke): SMOKE-core-01~05 本地 E2E runner（curl 打本地后端）
06c1007 fix(smoke): 修 SMOKE-core-01~05 runner 的 4 个具体 bug
```

## 验收

- `openlogos verify`：Gate 3.6 PASS（266 用例，210 pass / 0 fail / 56 skip）
- `openlogos smoke`：Gate 3.8 PASS（6 用例全 pass，Coverage 100%）
- `trunk build --release`：成功生成 `frontend-rs_bg.wasm` (1.05 MB)
- acceptance-report.md 已重新生成（2026-08-26）
- implementation-checklist §2.4 已勾选 DPR 行
- core-implementation-checklist 已更新

## 关联规格

- `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §11 HiDPI 渲染基线
- `logos/resources/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` §10.1/§10.2 web-font 契约
- `logos/resources/prd/2-product-design/1-feature-specs/core-0c-motion.md` §2.1 渲染稳定期
- `logos/resources/test/core-RP-canvas-hidpi-test-cases.md` UT-RP-01~05 / ST-RP-01~03