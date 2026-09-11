# Delta — core-S01-edit-and-save-diagram.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> Phase 2 输入：`core-S01-edit-and-save-design.md` | 原型：`core-01-editor-prototype.html`（唯一现行）
> 页面上下文：默认在 **`room-editor`** 内编辑；亦可兼容非 room 单人 diagram
> 生产状态：前后端已实现；与主原型壳层/状态文案的逐项对齐由 `implement-unified-prototype-spec-parity` 承接
> API/DB：本提案不新增端点或表；仅补前端参与者与异常映射

## ADDED — 进入路径（技术）

1. 用户经 S03 → S04 进入 `room-editor`（或打开既有 diagram）
2. Viewer：写工具禁用，**不**触发 debounce PUT
3. Owner/Editor：画布变更 → `editor_core.mark_dirty` → debounce PUT（非 OT）或 S05 op（协作模式）
4. 主原型仅演示保存态文案；生产以 `frontend-rs` REST/WS 为准

## MODIFIED — 2. 参与者

在既有表上追加 / 修正：

| 角色 | 模块 | 文件 / 锚点 |
|---|---|---|
| RoomBadge | AppBar | `[data-testid="room-badge"]`（可回 rooms） |
| StatusBar | editor_panels | `[data-testid="status-bar"]` / 角色 Tag |
| Inspector | editor_panels | `[data-testid="inspector"]`（与主原型一致；废止仅写 `inspector-panel` 为唯一名） |
| CodeView | code_view | `[data-testid="code-view-modal"]` · `[data-testid="btn-code-view"]` |
| ConflictModal | ModalRoot | `[data-testid="modal-conflict"]` — **仅非 OT 快照冲突路径** |

## MODIFIED — 5.1 409 Conflict（最复杂分支）

### 5.1 409 Conflict（最复杂分支）

**适用**：非 room 单人编辑，或 room 内未走 OT、仍以 revision 快照 PUT 发生冲突时。

**不适用（强制）**：S05 协作模式下服务器已 OT 合并的并发操作 → Toast / Activity 反馈，**禁止**弹出 `modal-conflict`。

```text
PUT → 409（非 OT）→ modal-conflict（reload / force / cancel）
S05 remote_op / CONFLICT_RESOLVED → Toast/Activity（无 409 模态）
```

## ADDED — 异常映射（前端）

| 后端 / 条件 | 前端反馈 | 锚点 |
|---|---|---|
| 200 + new revision | 「已保存」+ revision 递增 | `save-state` / `revision-display` |
| 409 revision_conflict（非 OT） | 冲突模态 | `modal-conflict` |
| 403 READ_ONLY（viewer） | Toast 只读；不入队 PUT | ToolRail 禁用 |
| 网络失败 | 「保存失败（离线）」+ 退避重试 | `save-state` |
| 401 token_expired | 交 S03 refresh 重放；用户无感知 | AuthClient interceptor |

## MODIFIED — 8. V1 边界

- ❌ 局部 PUT（仍全量）— 不变
- ~~❌ 实时协作同步（V1 仅单人；V2 OT）~~ → room 内协作见 **S05**；本场景保留非 OT 保存语义
- 非 room 单人编辑仍走本场景 PUT + 409

## MODIFIED — 9. 对齐参考源

- `core-01-editor-prototype.html` — AppBar / 保存态 / room-badge
- `core-S05-ot-collab.md` — 协作路径禁止 409 模态
- `core-00-information-architecture.md` — `room-editor` 页面状态
