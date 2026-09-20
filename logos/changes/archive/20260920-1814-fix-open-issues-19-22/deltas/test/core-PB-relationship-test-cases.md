# Delta — core-PB-relationship-test-cases.md（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#21、#22 | 规格：`core-01b-relationship.md` §4.1～§4.4

## ADDED — UT-PB-12 — 关系描边色优先级（#22）

- **位置**：`frontend-rs/src/editor_render.rs::relation_stroke_color`（签名扩展为可传入源表色）
- **断言**：
  - `ref.color` 非空 → 用关系色
  - `ref.color=""` 且源表色非空 → 用源表色
  - 二者皆空 → `palette.relation`
  - 选中外环仍用 `palette.selected`

## ADDED — UT-PB-13 — 正交折线消费选侧（#21）

- **位置**：`calc_path` / `calc_orthogonal_path`
- **GIVEN**：A→B，B 在 A 左侧；`line_type=orthogonal`
- **THEN**：路径为水平/垂直折线；起终点锚点与 `pick_port_sides` 一致（左出右进）；不回归固定右出左进

## ADDED — UT-PB-14 — 虚线线型（#21）

- **断言**：`stroke_style=dashed` → dash 数组非空；`solid` → 空 dash（实线）

## ADDED — UT-PB-15 — 选中态密度降噪（#21）

- **GIVEN**：三条关系，仅选中表 T（关联其中一条）
- **THEN**：与 T 无关的两条 alpha ≤ 0.25；相关一条 alpha = 1（或正常值）；无选中时全部正常

## ADDED — ST-PB-07 — e2e：正交折线持久化（#21）

- **WHEN**：选中关系 → Inspector 切换 `orthogonal` → 保存 → 刷新
- **THEN**：路径仍为折线；JSON 含 `line_type: orthogonal`

## ADDED — ST-PB-08 — e2e：出边跟随源表色（#22）

- **WHEN**：表设色且关系未设色 → 出边跟表色；关系显式设色后改表色
- **THEN**：显式色保留；清除关系色后恢复跟随

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。
