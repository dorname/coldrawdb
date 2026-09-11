# Delta — core-01b-relationship.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`（Tool Rail `tool-relationship` + 画布 `rel-rubber-band` / `rel-tool-hint`）。

| 项 | 约定 |
|---|---|
| 页面流 | 关系创建发生在 `room-editor`；入口为 Tool Rail，非独立历史原型 |
| 演示 ≠ 生产 | 主原型拖放/点击两点后**立即 commit**；生产须进入确认条再写入 |
| 实现状态 | **后端已实现**（reference）；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| 状态机命名 | 生产端关系工具状态含 **`RelToolState::Dragging`**（与 Idle / PickSource / PickTarget / Confirm 并列） |

## MODIFIED — 3.1 关系工具模式（Tool Rail `🔗`）

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

## MODIFIED — 3.2 关系确认条（非模态）

### 3.2 关系确认条（非模态）

- 生产：拖字段出线与点击两点**共用** `rel-confirm-bar`；默认 cardinality `one_to_many`；确认后写 `Reference`。
- 主原型：无确认条，第二次命中即 `commit`——验收生产时不得要求「与原型一样立即写入」。

## MODIFIED — 4. 渲染

- 表拖动与关系拖线均 `setPointerCapture`。
- 拖表过程中已有关系路径按未量化坐标在 rAF 中重算（见 `core-01`）。
- 生产松手网格 **`GRID_SIZE = 20`**（关系端点几何跟随表位置）。

## ADDED — §9.x Viewer / 只读

- Viewer：不得进入 `Dragging`；点击字段不得建立 `relationSource`。
- 只读分享（`share-readonly`）：同 Viewer，无关系工具。

## ADDED — §10.x 对齐参考补充

- 事实基线：`core-01-editor-prototype.html`（`DRAG_THRESHOLD`、`rel-rubber-band`、`schedulePaint`）
- 前序已合并：`optimize-canvas-connect-and-drag` 关系状态机与测试 ID（UT-PB-* / ST-PB-*）仍有效，本提案仅统一原型边界与实现状态措辞
