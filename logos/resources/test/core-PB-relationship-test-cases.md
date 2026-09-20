# core-PB-relationship-test-cases.md

> 模块：core | 提案：optimize-canvas-connect-and-drag
> 路径：`logos/resources/test/core-PB-relationship-test-cases.md`
> 最后更新：2026-08-22

## Phase B 关系工具测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PB-01 | 表含字段 (100,130) | `hit_test_field` | `Some((table_id, field_id))` |
| UT-PB-02 | draft 两端字段 | `build_reference` | `type_==one_to_many`, on_delete==RESTRICT |
| UT-PB-03 | reference A→B | `flip_reference_endpoints` | start/end 互换 |
| UT-PB-04 | 表两字段 f1 PK | `toggle_field_primary(f2)` | f2.primary=true, f1.primary=false |
| UT-PB-05 | 确认条可见 | 点 create | `references.len()+1` |
| UT-PB-06 | 源字段 pointerdown | 位移 3px 后 pointerup | 判定为点击，进入 `PickTarget`，无橡皮筋 |
| UT-PB-06B | 源字段 pointerdown | 位移 8px | 判定为 `Dragging`，出现 `rel-rubber-band` |
| UT-PB-07 | `Dragging` 源字段锚点 (x1,y1) | 指针 (x2,y2) | 橡皮筋 `d` 以 (x1,y1) 为起点、(x2,y2) 为终点 |
| ST-PB-01 | 两张表各一字段 | 关系工具**点击**两点直接落账（p0-fix 定点 3：无确认条） | Inspector 可编辑关系；`references.len()==1`；确认条不存在 |
| ST-PB-02 | 两张表各一字段 | 关系工具从字段 **pointerdown 拖到**另一字段，松开直接落账（p0-fix 定点 3） | `references.len()==1`；确认条不存在 |
| ST-PB-03 | 已有一条可见关系（两表拖开不重叠） | 点击连线中点 | 连线选中高亮；**不弹详情模态**（`modal-reference-detail` 不存在）；Inspector 关系面板可见且 label 为 `table_1.id → table_2.id`；点 Inspector「删除关系」→ 落账 `references.len()==0` |
| ST-PB-04 | 已有一条可见关系 | 点击连线选中 → 按 Delete 键；重建关系后再次选中 → 按 Backspace 键 | 两次均落账 `references.len()==0`（Delete 与 Backspace 双键覆盖） |

### UT-PB-06 — 点击 / 拖线阈值

- **位置**：`frontend-rs/src/editor_render.rs` 或 `editor_panels.rs` 纯函数（如 `is_relation_drag(dx, dy, threshold=4.0)`）
- **前置**：关系工具 `PickSource`
- **步骤**：
  1. `is_relation_drag(3, 0, 4)` → false
  2. `is_relation_drag(0, 4, 4)` → true
  3. `is_relation_drag(3, 3, 4)` → true（欧氏距离）
- **断言**：阈值比较使用指针位移的欧氏距离，单位为屏幕像素（再除以 zoom 前）

### UT-PB-07 — 橡皮筋路径端点

- **位置**：`frontend-rs/src/editor_render.rs`（`calc_path` 或 `rubber_band_path`）
- **前置**：源字段锚点已知
- **步骤**：用源锚点与指针坐标生成路径
- **断言**：路径起点等于源字段锚点；终点等于指针画布坐标；与正式关系线使用同一贝塞尔算法

### ST-PB-02 — 拖字段出线创建关系（e2e）

- **位置**：`frontend-rs/tests/e2e/16_relationship_tool.spec.ts`
- **步骤**：
  1. 创建两张表各一字段
  2. 点击 `tool-relationship`
  3. 在画布源字段 pointerdown，移动超过 4px，在目标字段 pointerup
  4. 生产端（p0-fix 定点 3 起）：松开**直接落账**，无确认条
- **断言**：Inspector 出现 1 条关系；点击两点用例 ST-PB-01 仍通过

## 统一原型对齐范围与状态

关系工具：`Dragging` 阈值 **4px**、rubber-band、点击两点；生产端原为确认条，**p0-fix 定点 3 起改为直接落账（确认条已删除）**。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## MODIFIED / ADDED — 用例

| ID | 变更 | 合同 |
|---|---|---|
| UT-PB-06 / 06B | MODIFIED | 位移 &lt;4px → 点击（PickTarget）；≥4px → `Dragging` + `rel-rubber-band`可见 |
| UT-PB-07 | MODIFIED | 橡皮筋 path 端点=源锚点→指针；与正式关系同算法 |
| ST-PB-01 | MODIFIED（p0-fix 定点 3） | 点击两点**直接落账**；确认条不存在 |
| ST-PB-02 | MODIFIED（p0-fix 定点 3） | 拖线松开**直接落账**；确认条不存在 |
| ST-PB-03（ADDED，p0-fix 定点 3） | ADDED | 点击连线中点 → `modal-reference-detail` 详情模态；模态内删除落账 0 条 |
| ST-PB-04（ADDED，p0-fix 定点 3） | ADDED | 选中连线按 Delete 键删除落账 0 条 |
| ~~ST-PB-CONFIRM~~ | REMOVED（p0-fix 定点 3） | 确认条交互已按提案删除，生产改为直接落账 |
| ST-PB-VIEWER（ADDED） | ADDED | Viewer 不得进入 Dragging / 建关系 |
| ST-PB-03/04（MODIFIED，relation-inspector-and-ddl-io） | MODIFIED | 点击连线不再弹详情模态（Inspector 为唯一详情通路）；删除收敛为 Inspector 按钮 + Delete/Backspace 双键 |

## 阈值纯函数（重申）

`is_relation_drag(dx,dy,threshold=4.0)` 使用欧氏距离；单位为屏幕像素（除 zoom 前）。

---

## 合并自 fix-remote-github-issues（2026-09-15）

## ADDED — UT-PB-08 — Idle 下字段拖连（无需激活关系工具）

### UT-PB-08 — 默认模式下字段→字段拖出建关系

- **位置**：`frontend-rs/src/editor_render.rs` / 关系手势状态机
- **前置**：`RelToolState::Idle`；两表各一字段；非 Viewer
- **步骤**：
  1. 源字段 pointerdown → 位移 ≥4px → 目标字段 pointerup
  2. 对照：位移 &lt;4px → 不建关系（字段选中）
- **断言**：
  - `references.len()==1`
  - ToolRail 关系按钮无需预先激活（`tool-relationship` 可保持未 pressed）
  - 写入通路走 `AddReference`（与 UT-KB-03 一致）

## ADDED — ST-PB-05 — 字段直连 e2e

### ST-PB-05 — 不点关系工具即可拖连

- **步骤**：不点击 `tool-relationship`，直接字段拖到另一字段
- **断言**：关系落账；既有 ST-PB-01 / ST-PB-02 仍通过

## MODIFIED — 用例表追加行

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PB-08 | Idle；两表字段 | 字段拖出 ≥4px 到另一字段 | 新增 1 条 reference；无需先点关系工具 |
| ST-PB-05 | 可写编辑器 | 不激活关系工具直接拖连 | 落账成功；ST-PB-01/02 回归通过 |

---

## 合并自 fix-remote-github-issues-7-18（2026-09-18）

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

### 用例登记（OpenLogos verify 解析用）

| ID | GIVEN | WHEN | THEN |
|---|---|---|---|
| UT-PB-09 | 两表几何（x / width） | `pick_port_sides` | 目标在左→左出右进；在右→右出左进；中心 x 相等→默认右出左进 |
| UT-PB-10 | 表 A(0,0) 与 B(-400,0) 各含字段 | `calc_path` | 起点==`A.x`、终点==`B.x+B.width`；c1<x1、c2>x2；B 移右侧回归右出左进 |
| UT-PB-11 | `ref.color` 非空 / 为空 | 关系描边取色纯函数 | 非空→该色；空→`palette.relation`；选中外环仍 `palette.selected` |
| ST-PB-06 | 画布含 ≥2 条关系 | Inspector 关系面板设预设色 → 保存 → 刷新 | 仅该线变色；刷新后仍在；导出 JSON 含 `color` 再导入保留 |

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。

## 合并自 fix-open-issues-19-22（2026-09-20）

## ADDED — UT-PB-12 — 关系描边色优先级（#22）

- **位置**：`relation_stroke_color`（可传入源表色）
- **断言**：显式色 > 源表色 > palette.relation

## ADDED — UT-PB-13 — 正交折线消费选侧（#21）

- **GIVEN**：A→B，B 在 A 左侧；`line_type=orthogonal`
- **THEN**：折线锚点与 `pick_port_sides` 一致（左出右进）

## ADDED — UT-PB-14 — 虚线线型（#21）

- **断言**：`dashed` → dash 非空；`solid` → 空 dash

## ADDED — UT-PB-15 — 选中态密度降噪（#21）

- **THEN**：非相关线 alpha ≤ 0.25；相关线正常；无选中时全部正常

## ADDED — ST-PB-07 — e2e：正交折线持久化（#21）

- **THEN**：切换 `orthogonal` → 保存刷新保留；JSON 含 `line_type`

## ADDED — ST-PB-08 — e2e：出边跟随源表色（#22）

- **THEN**：表设色且关系未设色 → 出边跟表色；显式设色优先

### 用例登记追加

| ID | 描述 |
|---|---|
| UT-PB-12 | 描边色优先级 |
| UT-PB-13 | 正交折线选侧 |
| UT-PB-14 | dashed/solid |
| UT-PB-15 | 密度降噪 alpha |
| ST-PB-07 | 正交折线持久化 |
| ST-PB-08 | 出边跟随源表色 |

## 合并自 layout-after-import-command（2026-09-20）

## ADDED — UT-PB-16 — 力导向布局确定性 / 无重叠 / 孤立不动（#23）

- **位置**：`frontend-rs/src/layout.rs::force_directed_layout`
- **参数**：iterations=100，spacing=180，seed=42
- **断言**：
  1. 相同输入两次结果完全一致（确定性）
  2. 连通表两两欧氏距离 > 50（无重叠）
  3. 无边表坐标不变（允许 jitter 扰动 < 5）
  4. 无边 / 空表：原样返回

## ADDED — ST-PB-09 — Command Palette「整理布局」触发（#23）

- **GIVEN**：画布含 ≥2 张连通表（有关系），坐标故意重叠或过近
- **WHEN**：打开 Command Palette → 选中 `palette-action-layout` / id=`action:layout`
- **THEN**：连通表 x/y 相对触发前发生变化；store dirty；孤立表（若有）坐标不变

### 用例登记追加

| ID | 描述 |
|---|---|
| UT-PB-16 | 力导向确定性 / 无重叠 / 孤立不动 |
| ST-PB-09 | palette 整理布局触发 |
