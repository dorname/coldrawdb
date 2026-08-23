# Delta — core-S01-edit-and-save-design.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头与原型策略

> 模块：core | 场景：S01 | 原型：`core-01-editor-prototype.html`（唯一现行主原型）
> Phase 1 输入：`core-04-scenario-detail.md` §S01
> 页面上下文：默认在 `room-editor` 内编辑；自动保存与 revision 反馈对齐 AppBar

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（浏览器端 ER 编辑器） |
| 原型形式 | 单文件可交互 HTML（内联 CSS/JS，不断网） |
| 对齐锚点 | 主原型 AppBar / ToolRail / Canvas / Inspector / StatusBar |
| 共享样式 | **不引用** `core-00-prototype-shared.css`；样式以内联主原型为准 |
| 生产边界 | 保存语义以 REST PUT + revision 为准；协作房间内并发合并见 S05，不得误用 409 模态 |

## MODIFIED — 2. 涉及页面与区域

| 区域 | coldrawdb 原型锚点 |
|---|---|
| 顶栏 | `[data-testid="app-bar"]` |
| 左侧工具 | `[data-testid="tool-rail"]` |
| 画布 | `[data-testid="editor-canvas"]` |
| 右侧 Inspector | `[data-testid="inspector"]` |
| 保存状态 | `[data-testid="save-state"]` / `[data-testid="revision-display"]` |
| 409 冲突（非 OT 路径） | `[data-testid="modal-conflict"]`（若生产仍保留） |
| Command Palette | `[data-testid="command-palette"]` · ⌘K / Ctrl+K |
| Code View | `[data-testid="code-view-modal"]` · `[data-testid="btn-code-view"]` |
| 房间上下文 | `[data-testid="room-badge"]`（可返回 rooms） |

## ADDED — 3.0 进入路径

1. 用户经 `auth → rooms` 进入 `room-editor`（或兼容的已打开 diagram）
2. Viewer 角色下写工具禁用，不触发 PUT
3. Owner/Editor 的画布变更进入 debounce 保存；协作模式下另见 S05 OT/ack

## MODIFIED — 3.3 409 revision 冲突

### 3.3 409 revision 冲突

仅适用于**非 OT 协作合并**的快照冲突路径。房间协作模式（S05）下服务器已合并的并发操作须 Toast/Activity 反馈，**禁止**弹出 S01 409 冲突模态。

## ADDED — 命令与代码视图（与主原型一致）

- Command Palette：搜索表/命令；Esc 关闭；关闭后焦点回画布
- Code View：SQL / DBML / JSON 分段切换；只读生成；复制；返回画布
- 二者不得破坏自动保存状态机；只读代码视图不产生 diagram mutation
