# 关系编辑规格（V1）

## 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`（Tool Rail `tool-relationship` + 画布 `rel-rubber-band` / `rel-tool-hint`）。

| 项 | 约定 |
|---|---|
| 页面流 | 关系创建发生在 `room-editor`；入口为 Tool Rail，非独立历史原型 |
| 演示 ≠ 生产 | 主原型拖放/点击两点后**立即 commit**；生产须进入确认条再写入 |
| 实现状态 | **后端已实现**（reference）；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| 状态机命名 | 生产端关系工具状态含 **`RelToolState::Dragging`**（与 Idle / PickSource / PickTarget / Confirm 并列） |

## 1. 关系（Relationship）对象结构

```ts
interface Relationship {
  id: string;
  name: string;              // 关系名（可选；如 "user_posts"）
  startTableId: string;      // 起点表 id
  startFieldId: string;      // 起点字段 id
  endTableId: string;        // 终点表 id
  endFieldId: string;        // 终点字段 id
  cardinality: "one_to_one" | "one_to_many" | "many_to_one" | "many_to_many";
  onUpdate: "CASCADE" | "RESTRICT" | "SET NULL" | "NO ACTION" | "SET DEFAULT";
  onDelete: "CASCADE" | "RESTRICT" | "SET NULL" | "NO ACTION" | "SET DEFAULT";
}
```

## 2. 关系类型语义

| cardinality | SQL 表达（以 MySQL 为例） | 含义 |
|---|---|---|
| `one_to_one` | `FOREIGN KEY (...) REFERENCES ... UNIQUE` | 1 : 1（双向唯一） |
| `one_to_many` | `FOREIGN KEY (...) REFERENCES ...` | 1 : N（外键在 N 端） |
| `many_to_one` | 同 `one_to_many`（方向相反） | N : 1 |
| `many_to_many` | 中间表 `link_<rel_name>` | N : N（需中间表） |

> `many_to_many` 实际实现：coldrawdb V1 在导出 SQL 时自动生成中间表 `link_<name>`（含两端外键）。中间表不在前端展示，但占用 `table_link` 关联。

## 3. 关系操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | **主要**：关系工具下从字段拖出连线到目标字段；**辅助**：依次点击源字段、目标字段 | 进入确认（生产）或写入 Relationship（原型立即写入）；自连线 = self-reference |
| 选中 / 查看详情 | 单击连线 | 选中该关系（`SelectionKind::Reference`）+ 打开 Inspector 关系面板展示详情（起止字段、cardinality、删除入口）；**不再弹详情模态**（relation-inspector-and-ddl-io：模态与 Inspector 信息重复，已移除） |
| 改 cardinality | 侧栏下拉 | 4 选 1 |
| 改 onUpdate / onDelete | 侧栏下拉 | 5 选 1 |
| 删除 | 选中后 Delete / Backspace，或 Inspector 关系面板「删除关系」按钮 | 从 diagram 移除 |
| 翻转 | 侧栏按钮 | 互换 start/end（不变 cardinality 含义） |

> 点击两点路径必须保留（ST-PB-01 / ST-PU-06）。位移小于 `DRAG_THRESHOLD`（4px）的 pointerdown/up 视为点击，不进入拖线。

### 3.1 关系工具模式（Tool Rail `🔗`）

**激活**：`tool-relationship` 或快捷键 `R`；Viewer 下按钮 disabled。

**关键规则（合同）**：

| 规则 | 规格 |
|---|---|
| 拖线阈值 | `DRAG_THRESHOLD = 4`px；位移 < 4px 视为点击，不进入 `Dragging`、不显示橡皮筋 |
| `Dragging` | 移动 ≥ 4px 后进入；`setPointerCapture`；橡皮筋 `data-testid="rel-rubber-band"` 可见；悬停目标字段高亮 |
| 松手 | 落在**不同**目标字段 → 生产进入 **Confirm**（确认条）；主原型立即写入。落空白/同源 → 保留源选中（PickSource），不写入 |
| 点击两点 | **必须保留**（ST-PB-01 / ST-PU-06）：第一次点源 → PickTarget；第二次点目标 → 生产 Confirm / 原型 commit |
| Esc / 取消工具 | 回 Idle；隐藏橡皮筋 |

**橡皮筋**：仅 `Dragging && moved` 时更新 `path[d]`；禁止每帧重建整页 DOM。

### 3.2 关系确认条（非模态）

- 生产：拖字段出线与点击两点**共用** `rel-confirm-bar`；默认 cardinality `one_to_many`；确认后写 `Reference`。
- 主原型：无确认条，第二次命中即 `commit`——验收生产时不得要求「与原型一样立即写入」。

## 4. 渲染

- 表拖动与关系拖线均 `setPointerCapture`。
- 拖表过程中已有关系路径按未量化坐标在 rAF 中重算（见 `core-01`）。
- 生产松手网格 **`GRID_SIZE = 20`**（关系端点几何跟随表位置）。

## 5. 校验

- 起点表 / 起点字段 / 终点表 / 终点字段必须存在
- 起点 ≠ 终点（**允许** self-reference：起点 = 终点）
- 字段类型匹配检查：V1 **不强制**（允许任意类型间建关系）
- onUpdate / onDelete 必填

## 6. 撤销 / 重做

关系的所有操作（创建 / 编辑 / 删除）都进入 `UndoRedoContext`。V1 不持久化。

## 7. 与后端实体的对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Relationship` | `reference` 表 | 1:1 映射 |
| cardinality | `reference.cardinality` 字段 | 枚举值 |
| onUpdate | `reference.on_update` | snake_case 存库 |
| onDelete | `reference.on_delete` | snake_case 存库 |
| startTableId + startFieldId | `reference.start_table_id` + `reference.start_field_id` | UUID |
| endTableId + endFieldId | `reference.end_table_id` + `reference.end_field_id` | UUID |
| many_to_many 中间表 | `table_link` 表 | 自动生成 |

## 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-R-01 | 创建 one_to_many 关系 |
| UT-R-02 | 改 cardinality 触发 SQL 重生成 |
| UT-R-03 | many_to_many 自动创建中间表 |
| UT-R-04 | 拖拽端点改变起止字段 |
| UT-R-05 | 删除关系不级联删除表 |
| ST-R-01 | 端到端：用户-文章一对多 → SQL 含正确外键 |
| UT-PB-01 | `hit_test_field` 命中字段行 |
| UT-PB-02 | `build_reference` 默认 RESTRICT |
| UT-PB-03 | `flip_reference_endpoints` 互换端点 |
| UT-PB-04 | `toggle_field_primary` 单表唯一 PK |
| UT-PB-05 | 确认条可见，点 create 后 `references.len()+1` |
| UT-PB-06 | 位移 < 4px 判定为点击；≥ 4px 判定为拖线 |
| UT-PB-07 | 橡皮筋路径起点为源字段锚点、终点为指针坐标 |
| ST-PB-01 | e2e：关系工具点击两点 + 确认 → Inspector 可编辑关系 |
| ST-PB-02 | e2e：关系工具从字段拖到另一字段 + 确认 → 新增 1 条 reference |

## 9. V1 边界

- ❌ 关系标签编辑（V1 不可编辑，drawdb 有）
- ❌ 关系颜色（V1 不可改）
- ❌ 关系线型（实线/虚线）（V1 统一实线）
- ❌ 关系的元数据（注释、tag）（V1 不支持）

### 9.1 Viewer / 只读

- Viewer：不得进入 `Dragging`；点击字段不得建立 `relationSource`。
- 只读分享（`share-readonly`）：同 Viewer，无关系工具。

## 10. 对齐参考源

- drawdb `src/components/EditorCanvas/Relationship.jsx`
- drawdb `src/components/EditorSidePanel/RelationshipsTab/RelationshipInfo.jsx`
- drawdb `src/utils/calcPath.js`（贝塞尔路径计算）
- coldrawdb `frontend-rs/src/editor_render.rs`（连线渲染）

---

### 10.1 对齐参考补充

- 事实基线：`core-01-editor-prototype.html`（`DRAG_THRESHOLD`、`rel-rubber-band`、`schedulePaint`）
- 前序已合并：`optimize-canvas-connect-and-drag` 关系状态机与测试 ID（UT-PB-* / ST-PB-*）仍有效，本提案仅统一原型边界与实现状态措辞

# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## 3 关系操作（E3 Tooltip / Popover 替换）

**merge 时在 §3 末尾追加**：

### §3.x 关系工具 Tooltip / Popover（E3）

关系工具激活时（`cdb-tool-rail-relationship` 按钮）显示：

| 元素 | E3 组件 | 内容 |
|---|---|---|
| 起点表 hover | `<Tooltip placement=Top>` | `"{table_name} ({field_count} 字段)"` |
| 起点字段 hover | `<Tooltip placement=Top>` | `"{field_name} : {field_type}"` |
| 终点表 hover | `<Tooltip placement=Top>` | `"{table_name}"` |
| 关系线 hover / 点击 | 选中 + Inspector 关系面板 | 关系详情：起点、终点、cardinality、onUpdate / onDelete（relation-inspector-and-ddl-io：居中详情模态已移除，详情唯一通路 = Inspector） |
| 关系线点击 | `<Popover>` 展开 | 同上 |

**视觉**：
- Tooltip：黑底白字，`--cdb-shadow-md`，`--cdb-radius-sm`
- Popover：白底，`--cdb-shadow-md`，`--cdb-radius-lg`，内容宽度 280px

**z-index**：Tooltip `--cdb-z-tooltip`（L4），Popover `--cdb-z-popover`（L4.5）

## 4 渲染（E2 关系线端点图标）

**merge 时在 §4 末尾追加**：

### §4.x 关系线端点图标（E2）

- 起点端点：`<IconKey />` 或 `<IconLink />`（小尺寸 12px）
- 终点端点：`<IconCaretDown />` 旋转 90°（one-to-many 视觉）
- 颜色：`var(--cdb-color-primary)`（默认）、`var(--cdb-color-warning)`（hover）
- 选中态：线粗 2.5px，色 `--cdb-color-primary-active`

---

## 合并自 fix-remote-github-issues（2026-09-15）

## MODIFIED — §3 关系操作 / 创建触发

| 操作 | 触发（更新后） | 数据变化 |
|---|---|---|
| 创建 | **A. 关系工具模式**（既有）：激活 `tool-relationship` 后拖字段或点击两点；**B. 字段直连手势（ADDED）**：在默认浏览/选择模式下，从字段行 pointerdown 拖出超过阈值到另一字段，无需先点击连线按钮 | 与既有确认/直接落账合同一致 |

## ADDED — §3.3 字段直连手势（Idle 下可拖）

**动机**：用户反馈「识别从字段到另一字段的连接，不需要单独点击连线按钮」。

| 规则 | 规格 |
|---|---|
| 入口 | `RelToolState::Idle`（或等价「未激活关系工具」）且非 Viewer / 非只读 |
| 起点 | 字段行命中（`hit_test_field`）pointerdown |
| 阈值 | 同 §3.1：`DRAG_THRESHOLD = 4`px；小于阈值 → 视为字段选中（不建关系） |
| 拖动中 | 显示 `rel-rubber-band`；可临时进入内部 `Dragging` 子态，**不必**点亮 ToolRail `tool-relationship` 按钮（按钮可保持未激活视觉） |
| 松手命中异表异字段 | 与关系工具拖线相同的落账通路（当前生产：直接写入 `Reference`） |
| 松手未命中 | 取消；不写入 |
| 与关系工具共存 | `R` / `tool-relationship` 点击两点与拖线合同**全部保留** |
| Undo | 必须走 `Command::AddReference`（见 core-KB UT-KB-03） |

## MODIFIED — Viewer / 只读

Viewer 与 share-readonly：**不得**启用字段直连手势（同既有关系工具门控）。

---

## 合并自 fix-remote-github-issues-7-18（2026-09-18）

## ADDED — §3.4 连线路径自动选侧（fix-remote-github-issues-7-18）

> 修复 issue #9：现行 `calc_path` 写死「源字段右点出、目标字段左点进」（`editor_render.rs` 中 `x1 = from.x + TABLE_WIDTH`、`x2 = to.x`），与双侧 port 的视觉暗示不一致。

**合同**：

| 规则 | 规格 |
|---|---|
| 出入侧选择 | 按两表相对几何位置自动选择：目标表中心 x < 源表中心 x → **左出右进**；否则 → **右出左进**。选择逻辑抽为纯函数（如 `pick_port_sides(from, to) -> (FieldPortSide, FieldPortSide)`），可单测 |
| 锚点 | 出/入锚点按所选侧取字段行左缘或右缘中点（复用 `field_anchor_for_side` 口径） |
| 贝塞尔控制点 | 控制点偏移方向跟随出入侧（左出时控制点向左伸展，右出向右），不得仍按固定 +x/-x 方向 |
| 拖拽建关系 | 字段行左右两侧 port 均可作为拖拽起点与放置终点（`hit_test_field_port` 命中的 side 必须被路径计算消费） |
| 动态更新 | 已有关系在任一端表拖动后按最新相对位置重算出入侧，无需用户重建关系 |
| 稳定性 | 两侧距离相等（x 差为 0）时保持现状默认（右出左进），避免边界抖动 |

**本版本仍不做**：自动绕障 routing、边捆绑、手动指定出入侧 UI。  
**本提案（layout-after-import-command / #23）新增**：一键整理布局与导入后自动整理（§4.5）。出入侧仍由 §3.4 `pick_port_sides` 自动决定。

## ADDED — §4.1 关系线颜色（fix-remote-github-issues-7-18；#22 继承语义）

> 实现 issue #12 的关系侧：`Reference` 增加可选 `color` 字段。#22 扩展空 color 继承源表色。

| 规则 | 规格 |
|---|---|
| 数据 | `Reference.color: string`（默认 `''` = 未配置）；前端模型、后端 `reference` 表（迁移 `0009_reference_color`）、`diagrams.yaml`、`mcp-tools.yaml` `UpdateReferenceInput` 同步扩展 |
| 渲染优先级 | 1) `Reference.color` 非空 → 关系显式色；2) 空且源表（`start_table_id`）`Table.color` 非空 → **跟随源表色**；3) 二者皆空 → `palette.relation` |
| 选中态 | 选中高亮优先级高于自定义色（选中态仍用 `palette.selected` 外环/加粗，自定义色作为基线色保留），二者不得互相覆盖导致不可辨识 |
| Inspector | 关系面板新增颜色入口（预设色板 + 清除回默认）；标明「默认 = 跟随源表」；变更走既有落账链路（store → dirty → schedule_save） |
| 导入导出 | JSON 导出保留 `color`、导入回填；SQL / DBML 导出**降级忽略**（不改变 DDL 语义），导入 SQL/DBML 时 `color=''` |
| 兼容 | 存量图无 `color` 字段 → 反序列化默认 `''`；无表色时视觉与主题默认色一致 |

## ADDED — §4.2 出边跟随源表色（fix-open-issues-19-22 / #22）

| 规则 | 规格 |
|---|---|
| 定义 | 「出边」= `start_table_id` 等于该表的 `Reference` |
| 即时性 | 源表 `color` 变更后，所有 `Reference.color==''` 的出边下一帧重绘即用新表色；**不得**因改表色静默覆盖已保存的非空 `Reference.color` |
| 兼容 | 存量图行为：无表色时仍为主题默认色，与 #12 交付观感一致 |

## ADDED — §4.3 线条类型与线型（fix-open-issues-19-22 / #21 C1）

| 规则 | 规格 |
|---|---|
| 数据 | `Reference.line_type`: `bezier`（默认）\| `orthogonal` \| `straight`；`Reference.stroke_style`: `solid`（默认）\| `dashed` |
| 持久化 | 前端模型 + 后端 `reference` 表（迁移 `0010_reference_line_style`）+ `diagrams.yaml` 同步；JSON 导入导出保留；SQL/DBML 降级忽略 |
| 路径 | `bezier` 保持现状三次贝塞尔；`straight` 为两端锚点直线；`orthogonal` 为水平/垂直折线（至少一折），锚点与出入侧由 §3.4 `pick_port_sides` 决定 |
| 线型 | `dashed` 使用 canvas `set_line_dash`；`solid` 实线 |
| Inspector | 选中关系后可切换线条类型与线型（`data-testid="inspector-rel-line-type"` / `inspector-rel-stroke-style`），走既有落账链路 |
| 兼容 | 缺字段反序列化为 `bezier` + `solid` |

## ADDED — §4.4 密度降噪（fix-open-issues-19-22 / #21 C2）

| 规则 | 规格 |
|---|---|
| 触发 | 画布选中 ≥1 张表或 ≥1 条关系时启用 |
| 相关定义 | 与选中表相连的关系、或选中关系本身，视为相关 |
| 视觉 | 非相关关系线不透明度降低（建议 ≤ 0.25），相关线保持正常或略加粗；不删除、不改数据 |
| 无选中 | 全部关系正常不透明度 |

## ADDED — §4.5 一键整理布局（layout-after-import-command / #23）

| 规则 | 规格 |
|---|---|
| 算法 | 复用 MCP `force_directed_layout`（Fruchterman-Reingold 变体）；前端 `frontend-rs/src/layout.rs` 纯函数；**不改 MCP** |
| 参数 | `iterations=100`，`spacing=180`，随机种子固定 `42`（确定性） |
| 输入 | 当前 `Table[]`（id/x/y）与 `Reference[]`（start_table_id / end_table_id） |
| 输出 | 仅改写连通表的 `x`/`y`；其他字段不变 |
| 孤立表 | 无关联边的表位置**不变** |
| 无边 | 关系为空时原样返回，不扰动坐标 |
| 命令入口 | Command Palette Action：id=`action:layout`，label=`整理布局`，`data-testid="palette-action-layout"` |
| 落账 | 写 store → dirty → schedule_save → 重绘（与拖表同链路） |
| 导入后自动 | 导入成功且**本次导入**解析出关系非空时，自动执行一次整理；关系为空不触发 |
| 不做 | 边捆绑、手动指定出入侧、绕障 routing |

## MODIFIED — 7. 与后端实体的对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Relationship` | `reference` 表 | 1:1 映射 |
| cardinality | `reference.cardinality` 字段 | 枚举值 |
| onUpdate | `reference.on_update` | snake_case 存库 |
| onDelete | `reference.on_delete` | snake_case 存库 |
| startTableId + startFieldId | `reference.start_table_id` + `reference.start_field_id` | UUID |
| endTableId + endFieldId | `reference.end_table_id` + `reference.end_field_id` | UUID |
| color | `reference.color` | 关系线颜色（`''` = 跟随源表色 / 主题默认）；迁移 `0009_reference_color` |
| line_type | `reference.line_type` | `bezier`/`orthogonal`/`straight`；迁移 `0010` |
| stroke_style | `reference.stroke_style` | `solid`/`dashed`；迁移 `0010` |
| many_to_many 中间表 | `table_link` 表 | 自动生成 |

## MODIFIED — 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-R-01 | 创建 one_to_many 关系 |
| UT-R-02 | 改 cardinality 触发 SQL 重生成 |
| UT-R-03 | many_to_many 自动创建中间表 |
| UT-R-04 | 拖拽端点改变起止字段 |
| UT-R-05 | 删除关系不级联删除表 |
| ST-R-01 | 端到端：用户-文章一对多 → SQL 含正确外键 |
| UT-PB-01 | `hit_test_field` 命中字段行 |
| UT-PB-02 | `build_reference` 默认 RESTRICT |
| UT-PB-03 | `flip_reference_endpoints` 互换端点 |
| UT-PB-04 | `toggle_field_primary` 单表唯一 PK |
| UT-PB-05 | 确认条可见，点 create 后 `references.len()+1` |
| UT-PB-06 | 位移 < 4px 判定为点击；≥ 4px 判定为拖线 |
| UT-PB-07 | 橡皮筋路径起点为源字段锚点、终点为指针坐标 |
| ST-PB-01 | e2e：关系工具点击两点 + 确认 → Inspector 可编辑关系 |
| ST-PB-02 | e2e：关系工具从字段拖到另一字段 + 确认 → 新增 1 条 reference |
| UT-PB-09 | `pick_port_sides` 按相对位置选侧（目标在左 → 左出右进；在右 → 右出左进；x 相等 → 默认） |
| UT-PB-10 | `calc_path` 消费选侧结果：锚点与贝塞尔控制点方向随侧变化 |
| UT-PB-11 | 关系 `color` 非空时渲染用色取自 `Reference.color`，为空回退源表色 / `palette.relation` |
| ST-PB-05 | e2e：目标表拖到源表左侧 → 连线改为左出右进，不再绕行 |
| ST-PB-06 | e2e：Inspector 修改关系颜色 → 仅该线变色 → 保存刷新后保留 |
| UT-PB-12 | `relation_stroke_color`：显式色 > 源表色 > palette |
| UT-PB-13 | `calc_path`/`calc_path_orthogonal`：orthogonal 折线锚点消费 `pick_port_sides` |
| UT-PB-14 | 线型 dashed 时 dash 数组非空；solid 为空 |
| UT-PB-15 | 选中表时非相关线 alpha 降低、相关线不降 |
| ST-PB-07 | e2e：切换正交折线 → 保存刷新保留 |
| ST-PB-08 | e2e：表设色且关系未设色 → 出边跟表色；关系显式设色后改表色不影响该线 |
| UT-PB-16 | 力导向：确定性 / 无重叠 / 孤立不动 |
| ST-PB-09 | Command Palette「整理布局」触发后连通表坐标变化 |

> 详细步骤见 `core-PB-relationship-test-cases.md`。
