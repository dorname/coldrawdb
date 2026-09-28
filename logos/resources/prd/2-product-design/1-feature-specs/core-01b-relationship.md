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

---

## 合并自 fix-open-issues-26-33（2026-09-27）

## ADDED — §4.6 批量修改全部关系线条类型（fix-open-issues-26-33 / #28）

| 规则 | 规格 |
|---|---|
| 入口 1 | Inspector 选中任一关系时，线条类型控件（`inspector-rel-line-type`）旁提供「应用到全部关系」按钮（`data-testid="inspector-rel-line-type-apply-all"`）：以当前选中关系正在选择的 `line_type` 值批量应用到图内全部关系 |
| 入口 2 | Command Palette 三个命令：`action:line-type-straight`（`palette-action-line-type-straight`，「将全部关系设为直线」）、`action:line-type-orthogonal`（正交）、`action:line-type-bezier`（贝塞尔曲线） |
| 范围 | 默认 = 当前图**全部关系**（含未选中）；无关系时入口禁用/命令 toast 提示「当前图无关系」 |
| 落账 | 批量修改在一次 CommandStack 事务内完成（多条 reference 的 `line_type` 更新合并为**单条命令**）→ dirty → schedule_save → 重绘，与单条改线型同一持久化链路；复用既有整体保存（PUT diagram），**不新增后端端点** |
| 撤销 | 一次 Undo 恢复批量前全部关系的 `line_type`（单条命令语义） |
| 只读 | 只读模式（分享只读 / 房间只读权限）入口禁用、命令不可触发 |
| 联动 | 本期仅批量 `line_type`；`stroke_style` 批量作为后续可选项，不在本变更范围 |
| 取值 | 与 §4.3 数据口径一致：`bezier` / `orthogonal` / `straight`；正交折线继续消费 §3.4 `pick_port_sides`，不回归 #9 选侧 |

**验收口径**：一键将约 20+ 条关系从 bezier 改为 straight，画布即时更新；保存刷新后保持；一次 Undo 全部还原；只读模式入口不可用。

## MODIFIED — §4.4 密度降噪（fix-open-issues-19-22 / #21 C2；fix-open-issues-26-33 / #31 强化）

| 规则 | 规格 |
|---|---|
| 触发 | 画布选中 ≥1 张表或 ≥1 条关系时启用 |
| 相关定义 | 与选中表相连的关系及其**对端表**（邻接表）、或选中关系本身及其两端表，视为相关；其余表/关系为非相关 |
| 选中表 | 选中描边线宽 ≥ 默认态 2×（或等效外发光/双层外环），颜色用 `palette.selected`；在密集画布（20+ 表）中选中表须一眼可辨 |
| 相关关系线 | 线宽 ≥ 默认 **1.5×**（建议 2×），全不透明度（alpha = 1），颜色沿用关系解析色（§4.1/§4.2）或 `palette.selected`，与非相关线形成明显明暗/粗细差 |
| 邻接表 | 保持正常亮度，可轻微强调（边框提亮）；不得被降透明度误伤 |
| 非相关关系线 | 不透明度 **≤ 0.25**；不删除、不改数据 |
| 非相关表 | 不透明度降低（建议 **≤ 0.5**）或等效变暗处理，退到背景层；hover/点击仍可正常选中（选中后即转为相关态恢复全亮） |
| 无选中 | 全部表/关系恢复正常不透明度与默认线宽 |
| 只读 | Viewer / 只读模式同样适用本视觉反馈（只读不阻止选中高亮，仅阻止编辑） |
| 性能 | 高亮态切换不得引入额外全量重光栅；alpha/线宽变化走既有逐帧重绘路径即可 |

**验收口径**：选中有 ≥3 条关系的表后，相关连线一眼可辨（粗细/明暗差显著）；非相关表/线明显退到背景层；取消选中后全图恢复正常对比度。

## 合并自 fix-31-related-line-emphasis（2026-09-28）

## MODIFIED — §4.4 密度降噪（fix-open-issues-19-22 / #21 C2；fix-open-issues-26-33 / #31 强化；fix-31-related-line-emphasis / #31 提亮）

| 规则 | 规格 |
|---|---|
| 触发 | 画布选中 ≥1 张表或 ≥1 条关系时启用 |
| 相关定义 | 与选中表相连的关系及其**对端表**（邻接表）、或选中关系本身及其两端表，视为相关；其余表/关系为非相关 |
| 选中表 | 选中描边线宽 ≥ 默认态 2×（或等效外发光/双层外环），颜色用 `palette.selected`；在密集画布（20+ 表）中选中表须一眼可辨 |
| 相关关系线 | 线宽 = 默认 **1.5×**（`RELATED_RELATION_WIDTH_FACTOR`，与 token `canvas.highlight.related-line-scale` 同源），alpha = 1；**主色强制 `palette.selected`**（不再沿用关系解析色 §4.1/§4.2），**光晕换用 `palette.selected_soft` 8px 发光**（选中关系自身为 3.5px 主线 + 10px 光晕，保持层级区分）；crow's foot 端点记号与主线同色（选中/相关态不得回落基线色）；与非相关线形成明显明暗/粗细差 |
| 邻接表 | 保持正常亮度，可轻微强调（边框提亮）；不得被降透明度误伤 |
| 非相关关系线 | 不透明度 **≤ 0.25**；不删除、不改数据 |
| 非相关表 | 不透明度降低（建议 **≤ 0.5**）或等效变暗处理，退到背景层；hover/点击仍可正常选中（选中后即转为相关态恢复全亮） |
| 无选中 | 全部表/关系恢复正常不透明度、默认线宽与关系解析色 |
| 只读 | Viewer / 只读模式同样适用本视觉反馈（只读不阻止选中高亮，仅阻止编辑） |
| 性能 | 高亮态切换不得引入额外全量重光栅；alpha/线宽/颜色变化走既有逐帧重绘路径即可 |

**验收口径**：选中有 ≥3 条关系的表后，相关连线以选中色族亮显并带发光光晕，与非相关线明暗/粗细差显著、一眼可辨；非相关表/线明显退到背景层；取消选中后全图恢复正常对比度。（fix-31-related-line-emphasis：针对 #31 重开反馈「提亮仍不明显」，相关线主色由关系解析色修订为 `palette.selected` 强制提亮，并同步端点记号着色。）

## 合并自 fix-issues-42-44-mcp-and-relation-hit（2026-09-28）

## ADDED — §4.7 关系线命中合同：线型同源 + 最近距离优先（fix-issues-42-44-mcp-and-relation-hit / #44）

**背景**：#44 命中错乱双根因——①`hit_test_reference_tier` 按 refs 数组序先命中即返回（非最近优先）；②命中几何恒为贝塞尔，忽略 `line_type`（orthogonal 折线绘制与命中不同源）。

合同条款：

- **R-HIT-01 线型同源**：命中检测几何必须与绘制几何同口径——`line_type=bezier` 用 `calc_path_tier` 贝塞尔（24 段折线近似）；`line_type=orthogonal` 用 `calc_orthogonal_path_tier` 折线顶点序列；tier（详情/拓扑）锚定策略一致（沿用 #35 R-LOD-08）。
- **R-HIT-02 最近距离优先（fix-issue-47 修订）**：命中带宽为 **8 屏幕像素**（实现上在世界坐标计算距离 `d`，按当前 `zoom` 换算：`d * zoom <= 8.0`）。带宽内若有多条关系命中，必须返回**点到线距离最小**的那条；禁止「数组序先命中即返回」。低 zoom 时不得“隔空命中”。
- **R-HIT-03 阈值外不命中**：所有关系距离均超阈值时必须返回 None——点击空白不得误选远处关系（与 §5.15 空白点击清选语义衔接）。
- **R-HIT-04 悬停/点击同口径**：点击选中与悬浮检测（§4.8）共用同一命中函数，保证「所见即所选」。
- **R-HIT-05 命中优先级不变**：关系线命中仍排在表命中之后（连线被表遮住时点击选中表）；便签/区域/字段端口优先级不变。

验收：UT-PB-18 / UT-PB-19；e2e ST-PB-11。

## ADDED — §4.8 悬浮关系线摘要 tooltip（fix-issues-42-44-mcp-and-relation-hit / #43）

**背景**：#43——字段维度下关系线密集、路径较长时，悬浮线段缺少「哪张表哪个字段 ↔ 哪张表哪个字段」的快速确认手段。

交互规格：

- **R-HOV-01 触发**：pointermove 且未处于任何拖拽态（表/关系/便签/区域/框选/pan/端点拖拽）时，对指针世界坐标执行 §4.7 命中；命中关系线即显示 tooltip。
- **R-HOV-02 内容**：单行摘要 `源表名.源字段名 → 目标表名.目标字段名`；表维度（拓扑档）下至少 `源表名 → 目标表名`（字段锚定仍存在时同样展示字段）。
- **R-HOV-03 位置（fix-issue-46 修订为 B1 固定角落摘要区）**：tooltip 为画布容器（`.cdb-canvas-stack`）内**左下角固定摘要区**（`position:absolute; left:12px; bottom:12px`），不跟随光标、不绑定任何指针坐标；不遮挡 Inspector；纯 DOM 浮层，不进画布绘制层。
  - 选型理由（issue #46 作者认可）：密集关系场景下固定角落稳定可预期、不挡线、不依赖线旁几何；坐标不来自指针事件，从根上消除「视口 client 坐标当容器局部坐标」的错位 bug 类。
  - 演进路径：待密集场景选中可靠性与线旁展示遮挡问题解决后，可评估 B2（线旁跟随）或 B3（混合降级）；本版锁定 B1。
- **R-HOV-04 无副作用**：悬停不改变选中态、不触发保存、不进撤销栈；点击仍按既有逻辑选中关系。
- **R-HOV-05 消失条件**：指针移出命中带宽、进入任意拖拽态、或命中目标变为非关系图元时消失。
- **R-HOV-06 只读/Viewer**：tooltip 同样可用（只读信息展示不受编辑权限限制）。

## §4.9 关系线选中高亮（fix-issue-47）

- **R-HL-REF-01 触发**：用户点击关系线后 `selected_ref_id` 置为该关系 id。
- **R-HL-REF-02 关系线高亮**：选中关系线以加粗/高亮色绘制（沿用 #31 选中关系线 3.5× 线宽）。
- **R-HL-REF-03 端点表高亮**：选中关系线时，其 `start_table_id` 与 `end_table_id` 对应的两张表同步进入视觉高亮状态（表边框/选中环与直接点表同口径），便于快速识别关联双方；不影响真实选中集合（不进入 `selected_table_ids`，Delete 等操作只作用于关系线）。
- **R-HL-REF-04 背景退化**：有任意选中（表/关系）时，非相关表/关系仍按 #31 规则 alpha 退化（0.25/0.5），保持视觉焦点。
- **R-PERF-HOV-01 信号瘦身**（#46）：hover 信号只携带 `ref_id`（Option<String>），不携带指针坐标——坐标每事件变化是抖动主因，移出信号后命中坐标不再驱动渲染。
- **R-PERF-HOV-02 写守卫**（#46）：pointermove 命中结果与当前 hover 值相同（同 ref_id）时**不得** set 信号；仅在「命中变化 / 命中消失 / 进入拖拽态清除」三种迁移时写信号。文案不变 → DOM 不重建。
- **R-PERF-HOV-03 命中节流**（#46）：hover 命中检测按 rAF 合并（每帧至多一次 `hit_test_reference_tier`），密集关系大图下避免每事件全量遍历 refs；拖拽/点击路径不节流（交互正确性优先）。
- **R-PERF-HOV-04 绘制解耦**（#46）：tooltip 为纯 DOM 节点，任何 hover 路径不得触发 `schedule_paint` / canvas 重绘；空闲（无悬停、无拖拽）时画布完全静止。

验收：UT-PB-20（文案纯函数）；e2e ST-PB-12（悬浮出 tooltip、选中态不变）。

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
| UT-PB-17 | 批量 line_type 命令纯函数与单次 Undo |
| ST-PB-10 | e2e：「应用到全部关系」+ 命令面板 |
| UT-PB-18 | 命中合同：阈值内最近距离优先；全超阈值返回 None（#44 R-HIT-02/03） |
| UT-PB-19 | 命中合同：线型同源——orthogonal 关系按折线几何命中，不按贝塞尔（#44 R-HIT-01） |
| UT-PB-20 | 悬浮 tooltip 文案纯函数：`源表.源字段 → 目标表.目标字段`；拓扑档 `源表 → 目标表`（#43 R-HOV-02） |
| ST-PB-11 | e2e：密集近平行线连续点击不同线段，选中与点击目标一致；两线中间空白不误选（#44） |
| ST-PB-12 | e2e：悬浮关系线出 tooltip 且选中态不变；移出消失（#43） |

> 详细步骤见 `core-PB-relationship-test-cases.md`。
