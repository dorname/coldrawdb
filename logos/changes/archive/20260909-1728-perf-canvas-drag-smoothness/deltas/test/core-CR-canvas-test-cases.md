## ADDED — 位置：`### ST-CR-INSP-01` 条目之后、`### 附录 A 追加`（第 298 行处）之前

### UT-CR-SPRITE-01 — 表精灵指纹与 zoom 分档（R-PERF-07）

- **位置**：`frontend-rs/src/editor_render.rs`（`sprite_zoom_bucket` / `table_sprite_fingerprint` 纯函数）
- **步骤**：构造同一 `Table` 的多份变体，分别计算指纹
- **断言**：
  - 分档边界：zoom 0.5 / 1.0 → 档 1；1.01 / 2.0 → 档 2
  - 坐标稳定性：仅改 `x` / `y` 指纹不变（拖动不触发重光栅）
  - 失效面：改名 / 改字段类型 / 改主键 / 改色 / 改宽 / 主题 / dpr×100 / 分档 任一变化指纹必变

### UT-CR-GUARD-01 — DOM 写守护（R-PERF-08）

- **位置**：`frontend-rs/src/editor_render.rs`（`dom_write_guard` 纯函数）
- **断言**：空→空 不写；值变化写并更新 last；同值连帧不写；新值再写；清空也算变更

### UT-CR-DPR-01 — 拖拽降采样 dpr（R-PERF-10）

- **位置**：`frontend-rs/src/editor_render.rs`（`capped_drag_dpr` 纯函数）
- **断言**：
  - 拖拽活跃：dpr 2.0 / 3.0 / 1.5 均压到 `DRAG_RENDER_DPR_CAP = 1.0`
  - 低 dpr 设备（0.8）不再降（min 语义，不抬升）
  - 非拖拽：原样透传（2.0 → 2.0，1.0 → 1.0）

### UT-CR-GHOST-01 — 幽灵层准入判定与 CSS 定位（R-PERF-11）

- **位置**：`frontend-rs/src/editor_render.rs`（`table_has_references` / `ghost_css_position` 纯函数）
- **断言**：
  - refs 起点表 / 终点表判定为有关联（禁走幽灵层）；无关联表与空 refs 可走幽灵层
  - 定位换算 `(x - margin) × zoom + pan_x` / `(y - margin) × zoom + pan_y`，原点零值用例

### ST-CR-GHOST-01 — 无关系线表拖拽期间主画布零重绘（e2e）

- **位置**：`frontend-rs/scripts/test-canvas-perf.mjs` + `window.__cdb_paint_count` debug 计数
- **步骤**：建 6 张无关系线表 → 拖动首表（≥20 次 pointermove）→ 松手
- **断言**：
  - 拖动中出现 `data-testid="drag-ghost"` 幽灵层节点，松手后被移除
  - 拖动 move 阶段 `__cdb_paint_count` 增长 ≤ 2（仅幽灵层创建当帧的「抠出」重绘；零重绘路径不再逐帧 paint）
  - 松手落账后画布恢复正常重绘（paint_count 恢复增长）且表位置与拖动终点一致

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-SPRITE-01 | 表精灵指纹与 zoom 分档 | `editor_render.rs::sprite_zoom_bucket` / `table_sprite_fingerprint` |
| UT-CR-GUARD-01 | DOM 写守护 | `editor_render.rs::dom_write_guard` |
| UT-CR-DPR-01 | 拖拽降采样 dpr | `editor_render.rs::capped_drag_dpr` |
| UT-CR-GHOST-01 | 幽灵层准入判定与 CSS 定位 | `editor_render.rs::table_has_references` / `ghost_css_position` |
| ST-CR-GHOST-01 | 无关系线表拖拽主画布零重绘 | e2e test-canvas-perf + `__cdb_paint_count` + `drag-ghost` |

## MODIFIED — ### ST-CR-PAN-01 — pan 走 rAF 合并（e2e，可选 debug 计数）

- **位置**：`frontend-rs/scripts/`（复用 test-spec-parity 风格脚本）+ `window.__cdb_paint_count` debug 计数
- **步骤**：滚轮缩放 / 空格拖拽平移连续触发 ≥ 10 次事件，统计 `draw_canvas` 实际执行次数
- **断言**：
  - 同一帧内多次事件至多 1 次重绘（paint_count 增长 ≤ 事件帧数）
  - **perf-canvas-drag-smoothness 口径修订**：平移断言放宽为 `≤ 事件数 + 1`——R-PERF-10 拖拽降采样在松手帧恢复全分辨率 backing store，产生一次有界的收尾重绘；rAF 合并不变量（每帧至多 1 次重绘、与事件数线性有界）不变
  - 缩放后光标下对象仍在光标附近（锚定 e2e 佐证，容差 ±2 CSS px）
