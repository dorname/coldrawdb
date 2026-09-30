# Delta — core-PB-relationship-test-cases.md（修改）

> merge 时按 ADDED 标记合并到 `logos/resources/test/core-PB-relationship-test-cases.md`

> 模块：core | 提案：fix-relation-mouse-hit-precision

## ADDED — fix-relation-mouse-hit-precision 端点记号命中热区

### UT-PB-24 — 端点记号外侧点命中关系

- **位置**：`frontend-rs/src/editor_render.rs`（`dist_to_reference` / `hit_test_reference_tier`）
- **前置**：构造关系线 t1(100,130) → t2(600,130)，字段 `f1`/`f2`，`line_type` 分别取 `bezier` / `orthogonal` / `straight`
- **步骤**：
  1. 计算关系路径端点中心 `(x1,y1)`、`(x2,y2)`
  2. 在右端点中心 `(x2,y2)` 外侧取点：垂直偏移 +9px（距主线 9px，超出 8px 主线带宽；距端点中心 9px，在 10px 端点热区内）
  3. 对三种 `line_type` 分别调用 `hit_test_reference_tier(..., zoom=1.0)`
  4. 再取距端点中心 11px 的外侧点作为对照
- **断言**：
  - 步骤 2 三种线型均命中 `r1`（端点热区生效）
  - 步骤 4 三种线型均返回 `None`（超出 10px 端点热区）
  - 端点热区与主线距离取 min：某点距主线 12px 但距端点中心 6px 时仍命中

### ST-PB-14 — 点击端点记号附近 10px 热区选中关系

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 1 张表 t1(100,130)、t2(600,130)，字段 `f1`/`f2`，关系 `r1`（默认 `line_type` bezier，`type_` one_to_many）
- **WHEN**：进入房间，点击关系线右端端点附近 10px 热区（坐标为右端点中心垂直向下偏移 9px，超出 8px 主线带宽）
- **THEN**：
  1. `sel_ref_id === "r1"`（命中探针）
  2. 关系线进入高亮渲染状态
  3. 点击空白处后 `sel_ref_id === null`
- **reporter**：`ST-PB-14` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-PB-24 | 端点记号命中热区 | `editor_render.rs::dist_to_reference` |
| ST-PB-14 | 点击端点记号附近热区选中关系 | e2e canvas endpoint hit |