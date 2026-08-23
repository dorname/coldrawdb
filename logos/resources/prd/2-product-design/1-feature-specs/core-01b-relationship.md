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
| 编辑 | 双击连线 | 打开 `RelationshipInfo` 侧栏面板 |
| 改 cardinality | 侧栏下拉 | 4 选 1 |
| 改 onUpdate / onDelete | 侧栏下拉 | 5 选 1 |
| 删除 | 选中后 Delete | 从 diagram 移除 |
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
| 关系线 hover | `<Popover trigger=Click>` | 关系详情：起点、终点、cardinality、onUpdate / onDelete（来自 main `RelationshipInfo.jsx`） |
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
