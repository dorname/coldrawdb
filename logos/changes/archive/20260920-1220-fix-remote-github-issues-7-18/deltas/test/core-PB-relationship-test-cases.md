# Delta — core-PB-relationship-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#9（连线自动选侧）、#12（关系颜色） | 规格：`core-01b-relationship.md` §3.4 / §4.1

## ADDED — UT-PB-09 — `pick_port_sides` 按相对位置选侧

- **位置**：`frontend-rs/src/editor_render.rs`（新增纯函数 `pick_port_sides`）
- **断言**：
  - 目标表中心 x < 源表中心 x → `(左出, 右进)`
  - 目标在右 → `(右出, 左进)`
  - 中心 x 相等 → 默认 `(右出, 左进)`（稳定性口径）
  - 输入仅依赖两表几何（x / width），与字段无关

## ADDED — UT-PB-10 — `calc_path` 消费选侧结果

- **位置**：`frontend-rs/src/editor_render.rs::calc_path`
- **GIVEN**：表 A(0,0) 与表 B(-400,0)（B 在 A 左侧），各含字段
- **WHEN**：计算 A→B 路径
- **THEN**：起点 x == `A.x`（左缘），终点 x == `B.x + B.width`（右缘）；贝塞尔控制点 c1 < x1（向左伸展）、c2 > x2（向右伸展）；B 移到 A 右侧后回归右出左进口径

## ADDED — UT-PB-11 — 关系线颜色回退

- **位置**：`frontend-rs/src/editor_render.rs`（关系描边取色纯函数）
- **断言**：`ref.color = "#f0a"` → 描边为该色；`ref.color = ""` → `palette.relation`；选中态外环仍取 `palette.selected`（不被自定义色覆盖）

## ADDED — ST-PB-05 — e2e：目标表左移后连线换侧

- **GIVEN**：画布含 A→B 一条关系，B 初始在 A 右侧（连线右出左进）
- **WHEN**：拖动 B 到 A 左侧并松手落账
- **THEN**：连线路径 d 更新为左出右进（起点为 A 左缘锚点）；无大范围绕行弧线；刷新后关系仍在

## ADDED — ST-PB-06 — e2e：关系颜色配置与持久化

- **GIVEN**：画布含 ≥2 条关系
- **WHEN**：选中其一，Inspector 关系面板设置颜色为预设色 → 保存 → 刷新
- **THEN**：仅该关系线变色（`stroke` 生效），其他关系不变；刷新后颜色仍在；导出 JSON 含该 `color`，再导入保留

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。
