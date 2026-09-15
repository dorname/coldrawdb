# Delta — core-01b-relationship.md（#3 无需先点关系工具即可字段拖连）

> 变更：fix-remote-github-issues / GitHub #3

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
