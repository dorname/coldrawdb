# Delta — core-01b-relationship.md

> 模块：core | 提案：optimize-canvas-connect-and-drag

## MODIFIED — 3. 关系操作

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

**激活**：点击 Tool Rail `tool-relationship` 或快捷键 `R`；按钮进入 `cdb-is-active` 态。

**状态机**：

| 状态 | 用户操作 | 下一状态 |
|------|----------|----------|
| `Idle` | 激活关系工具 | `PickSource` |
| `PickSource` | 在字段上 pointerdown 且移动 ≥ 4px | `Dragging` |
| `PickSource` | 点击源字段（移动 < 4px） | `PickTarget` |
| `Dragging` | pointermove | 保持 `Dragging`；橡皮筋端点跟随指针；悬停目标字段高亮 |
| `Dragging` | 在**不同**目标字段 pointerup | 生产：`Confirm`；原型：立即写入关系并回 `Idle` |
| `Dragging` | 松在空白、同源字段或无效命中 | `PickSource`（不写入） |
| `PickTarget` | 点击目标字段 | 生产：`Confirm`；原型：立即写入关系并回 `Idle` |
| `PickTarget` | 从已选源字段再次按下并拖出 | `Dragging` |
| `Confirm` | 点「创建」 | `Idle`（关系写入 store） |
| `Confirm` | 点「取消」 | `PickSource` |
| 任意 | 按 `Esc` 或切回选择工具 | `Idle` |

**橡皮筋预览**（`data-testid="rel-rubber-band"`）：
- 仅在 `Dragging` 且已超过 4px 阈值时可见
- 起点 = 源字段行锚点（表右侧中线）；终点 = 指针的画布坐标
- 路径算法与正式关系线相同（`calc_path` / 原型贝塞尔）
- 不得每帧重建整页 DOM；只更新该 `<path>` 的 `d`

**画布提示**（`data-testid="rel-tool-hint"`）：
- `PickSource`：「从字段拖出连线，或点击选择源字段」
- `Dragging`：「拖到目标字段后松开」
- `PickTarget`：「选择目标字段，或从源字段拖出连线」

**Viewer / 只读**：关系工具 disabled；按下字段不得进入 `Dragging`。

### 3.2 关系确认条（非模态）

**位置**：画布底部居中（`z-index: L2`，不遮挡 AppBar / Inspector）。

```
┌──────────────────────────────────────────────────────┐
│ users.id → orders.user_id   [1:N ▼]  [创建] [取消]   │
└──────────────────────────────────────────────────────┘
```

- testid：`rel-confirm-bar` / `rel-confirm-create` / `rel-confirm-cancel` / `rel-confirm-cardinality`
- 默认 cardinality：`one_to_many`
- 创建后：写入 `Reference`，`type_` = cardinality，`on_delete`/`on_update` 默认 `RESTRICT`
- 拖字段出线与点击两点**共用**本确认条（生产前端）
- 主原型为单文件演示、无确认条：第二次点击或拖放到目标字段后立即 `commit` 写入关系（对齐 ST-PU-06）

## MODIFIED — 4. 渲染

- **连线**：贝塞尔曲线（`BezierCurve` 算法，详见 `frontend-rs/editor_render` 中的 `calc_path`）
- **端点**：箭头（drawdb 用 SVG marker；coldrawdb V1 用 canvas 自绘）
- **标签**：起点端 cardinality 标签 + 终点端 cardinality 标签
- **路径重算**：起点 / 终点表在**拖动过程中**按未量化的视觉坐标每动画帧重算并重绘；禁止等到 pointerup 才更新连线
- **生产端绘制**：`requestAnimationFrame` 合并；拖动中使用临时坐标，不得每 pointermove `store.tables.set(整表数组)`
- **指针捕获**：表拖动与关系拖线均 `setPointerCapture`，光标离开画布不得丢事件
- **碰撞**：连线与表头有最小间距，绕开

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
