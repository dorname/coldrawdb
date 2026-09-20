# Delta — core-CR-canvas-test-cases.md（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#19、#20、#22 | 规格：`core-01-editor-canvas.md` §5.9/§5.10、`core-01a` §1.6/§1.7

## ADDED — UT-CR-DRAG-01 — 有效拖动后抑制 click 穿透（#19）

- **位置**：`frontend-rs/src/editor_render.rs`（拖动结束 suppress 标志 / 纯函数 `should_suppress_click_after_drag`）
- **断言**：
  - 位移 ≥ `DRAG_THRESHOLD` → 返回 true（应抑制下一次 click）
  - 位移 < 阈值 → false（点击选中与顶栏 click 正常）
  - 区域 / 表 / 便签三类拖动共用同一判定

## ADDED — ST-CR-DRAG-01 — e2e：区域拖到邀请按钮不打开模态（#19）

- **GIVEN**：协作房间编辑器，画布有区域，AppBar 可见 `btn-invite`
- **WHEN**：对区域 `pointerdown` → 指针移到 `btn-invite` 中心 → `pointerup`
- **THEN**：`modal-invite` **不出现**；区域坐标已更新（与拖动位移一致）

## ADDED — UT-CR-WIDTH-01 — auto / 固定宽分支（#20）

- **位置**：`frontend-rs/src/editor_render.rs::compute_table_render_size`（或 `resolve_table_width`）
- **断言**：
  - `width=None` 与 `width=Some(0)` → auto 路径，结果 ∈ `[230, 480]`
  - `width=Some(350)` → 有效宽 350（不夹到内容）

## ADDED — UT-CR-WIDTH-02 — 长表名撑宽（#20）

- **GIVEN**：表名 `asset_v2.virtualization_cluster_profile`，`width=None`，注释模式 `name`
- **THEN**：有效宽 > 230 且 ≤ 480；短名表（如 `t`）可为 230

## ADDED — ST-CR-WIDTH-01 — e2e：长表名可读（#20）

- **GIVEN**：导入或创建含长表名的图
- **THEN**：画布表头可见完整或显著宽于默认卡；Inspector 名称完整；刷新后宽行为符合 auto/手动设定

## ADDED — UT-CR-COLOR-02 — picker 写入任意色（#22）

- **位置**：表色更新通路（纯函数或 store 写入）
- **断言**：合法 hex（如 `#a1b2c3`）写入 `table.color` 后 `table_border_color` 派生自该值；清空 `''` 回退 palette

## ADDED — ST-CR-COLOR-02 — e2e：picker + 出边跟随（#22）

- **GIVEN**：表 A 有 ≥1 条出边且关系 `color=''`
- **WHEN**：Inspector 用 color picker 设表色 → 观察出边 → 保存刷新；再给该关系显式设色后改表色
- **THEN**：未设色出边跟随表色；显式设色后改表色不影响该线；清除关系色后恢复跟随

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。
