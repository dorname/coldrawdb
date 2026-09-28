# Delta — core-PB-relationship-test-cases.md（fix-issue-47-relation-hit-precision-and-highlight）

## ADDED — UT-PB-22 — 命中阈值屏幕像素等价（zoom 感知）

- **类型**：前端纯函数测试（`frontend-rs` lib）
- **覆盖**：`hit_test_reference_tier`
- **断言**：
  1. zoom=1.0 时，点距线 8 世界像素命中、9 世界像素不命中（基线）
  2. zoom=0.5 时，点距线 16 世界像素命中（16×0.5=8 屏幕像素）、17 世界像素不命中——证明阈值按屏幕像素等价缩放
  3. zoom=2.0 时，点距线 4 世界像素命中、5 世界像素不命中
  4. 最近距离优先逻辑不变（沿用 UT-PB-18）
- reporter 登记 `UT-PB-22`

## ADDED — UT-PB-23 — 关系线选中时端点表视觉高亮

- **类型**：前端锚点/纯函数测试（`frontend-rs` lib）
- **覆盖**：`draw_canvas` 中表 `is_sel` 计算或新增高亮判定
- **断言**：
  1. 存在纯函数/逻辑：给定 `selected_ref_id` 与 refs，可判定某 table_id 是否为选中关系的端点表
  2. `draw_canvas` 调用 `draw_table(..., selected=true, ...)` 对端点表生效（源码锚点）
  3. 非端点表在该状态下不被误标为 selected
- reporter 登记 `UT-PB-23`

## ADDED — ST-PB-13 — 点击关系线高亮关系线与两端表 e2e

- **类型**：e2e（`frontend-rs/scripts/test-spec-parity-d.mjs`）
- **步骤**：
  1. 预置两表一关系（t1.f1 → t2.f2）
  2. 点击关系线中段 → 断言 `sel_ref_id === "r1"`（hl 探针）
  3. 断言关系线处于高亮渲染状态（探针 `rel_hl_ref_id === "r1"` 或等效字段）
  4. 断言两端表 `t1`、`t2` 均进入高亮/selected 视觉状态（通过 DOM class 或 canvas 探针）
  5. 点击空白 → 高亮清除
- reporter 登记 `ST-PB-13`

## MODIFIED — 用例 ID 清单（附录）追加

| ID | 标题 |
|---|---|
| UT-PB-22 | 命中阈值屏幕像素等价（zoom 感知，fix-issue-47） |
| UT-PB-23 | 关系线选中时端点表视觉高亮 |
| ST-PB-13 | 点击关系线高亮关系线与两端表 e2e |
