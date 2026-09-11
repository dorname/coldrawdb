# core-RP-canvas-hidpi-test-cases — 画布 HiDPI 渲染测试

> 模块：core | 提案：fix-canvas-hidpi-rendering | created: 2026-08-26
> 关联规格：`core-01-editor-canvas.md` §11、`core-07-design-tokens.md` §10.1/§10.2、`core-0c-motion.md` §2.1

## 0. 范围

画布 backing store 在 devicePixelRatio=1/1.5/2 下尺寸、字体加载契约、首帧时序与视觉回归。

## 1. 单元测试

### UT-RP-01 — DPR=1 backing store 等于 CSS 像素
- **GIVEN** `window.devicePixelRatio = 1`，父容器 clientWidth=1200 / clientHeight=800
- **WHEN** `setup_canvas_resize_observer` 触发
- **THEN** `canvas.width = 1200`、`canvas.height = 800`

### UT-RP-02 — DPR=2 backing store 等于 2× CSS 像素
- **GIVEN** `window.devicePixelRatio = 2`，父容器 clientWidth=1200 / clientHeight=800
- **WHEN** `setup_canvas_resize_observer` 触发
- **THEN** `canvas.width = 2400`、`canvas.height = 1600`

### UT-RP-03 — zoom 累乘保护
- **GIVEN** `dpr=2`、`transform.zoom=1.0`
- **WHEN** 用户滚轮 zoom 至 2.0；再 zoom 至 0.5
- **THEN** 当前 `ctx.getTransform()` 矩阵的 a/b/c/d = `[1.0, 0, 0, 1.0]`（每帧 `set_transform` 复位），不出现 `[2,0,0,2]` 后变 `[4,0,0,4]`

### UT-RP-04 — matchMedia DPR 变化触发 redraw
- **GIVEN** `matchMedia('(resolution: 1dppx)')` 监听挂载
- **WHEN** 模拟 DPR 从 1 → 2（如拖动窗口到外接显示器）
- **THEN** redraw 计数 ≥ 1，canvas 像素切换为 2× CSS

### UT-RP-05 — font-ready 3s 超时兜底
- **GIVEN** Google Fonts 网络被屏蔽（fetch 永远 pending）
- **WHEN** 应用启动
- **THEN** 3000ms 后首帧仍能提交（不阻塞），`CANVAS_FONT` 已降级到 `ui-monospace`

## 2. 场景测试（playwright e2e）

### ST-RP-01 — DPR=1 / DPR=2 像素相似度 ≥ 95%
- **GIVEN** `chromium.launch({ args: ['--force-device-scale-factor=1'] })` 与 `2` 两组
- **WHEN** 加载 `/?share=<sample>`，等待 `document.fonts.ready`
- **THEN** 截取 `.cdb-canvas-container` 区域，与基线 `canvas-baseline-dpr{1,2}.png` 比较，像素相似度 ≥ 95%

### ST-RP-02 — web font 失败不阻塞首帧
- **GIVEN** `page.route('**/fonts.googleapis.com/**', route => route.abort())`
- **WHEN** 加载 `/`
- **THEN** 3.5s 内画布出现内容（首帧已提交），无空白屏

### ST-RP-03 — HP-01~HP-05 smoke 仍 PASS
- **GIVEN** dpr=1 默认
- **WHEN** 跑 `scripts/smoke-local-scripts.sh`
- **THEN** HP-01~HP-05 全 pass

## 3. 验收条件

| ID | 验收 |
|---|---|
| AC-RP-01 | `frontend-rs/src/editor_render.rs::setup_canvas_resize_observer` 实现 DPR 感知 |
| AC-RP-02 | `lib.rs::mount` 在 `document.fonts.ready` resolve 后才调度首帧 |
| AC-RP-03 | `index.html` Google Fonts URL 含 `display=optional` |
| AC-RP-04 | `styles.css :root` 新增 `text-rendering: optimizeLegibility` |
| AC-RP-05 | 56 skip 全部转入 e2e harness（见 `complete-skipped-e2e`）后 `openlogos verify` 全 pass |

## 4. 验收执行

```bash
cd frontend-rs
trunk build --release
cargo test --target wasm32-unknown-unknown  # 仅 UT-RP-*（e2e 在另仓跑）
cd ..
bash scripts/run-verify-tests.sh           # 阶段 2 e2e harness
openlogos verify                            # 期望 all pass
openlogos smoke                             # 期望 SMOKE_PASS
```