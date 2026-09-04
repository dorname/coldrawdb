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
| ST-PB-03 | 已有一条可见关系（两表拖开不重叠） | 点击连线中点 → 弹关系详情模态 → 点「删除关系」 | `modal-reference-detail` 可见且 label 为 `table_1.id → table_2.id`；删除后落账 `references.len()==0` |
| ST-PB-04 | 已有一条可见关系 | 点击连线选中 → 关闭详情模态 → 按 Delete 键 | 落账 `references.len()==0` |

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

## 阈值纯函数（重申）

`is_relation_drag(dx,dy,threshold=4.0)` 使用欧氏距离；单位为屏幕像素（除 zoom 前）。
