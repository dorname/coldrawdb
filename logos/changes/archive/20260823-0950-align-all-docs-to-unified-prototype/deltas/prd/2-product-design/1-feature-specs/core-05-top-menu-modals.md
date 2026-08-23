# Delta — core-05-top-menu-modals.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。AppBar / 菜单 / 模态以 `room-editor` 顶栏为准。

| 项 | 约定 |
|---|---|
| 页面流 | 用户经 `auth → rooms` 进入编辑器；`room-badge` 可返回 rooms |
| 演示 ≠ 生产 | 邀请复制、分享链接、会话续期、原型诊断可为模拟；生产以 auth/rooms API 为准 |
| 实现状态 | **后端已实现**；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |

## ADDED — 统一原型对齐补充：1. 顶部菜单布局（V2 — R4 信息分层）

```
[品牌] Logo · Undo/Redo · diagram-title · room-badge · save-state
                                    …spacer…
[操作] presence · invite · 成员 · code · more · user-menu
```

| 元素 | `data-testid` | 说明 |
|---|---|---|
| 顶栏容器 | `app-bar` | 玻璃态 AppBar |
| 撤销 / 重做 | `btn-undo` / `btn-redo` | Viewer 或空栈时 disabled |
| 标题 | `diagram-title` | 可编辑；Viewer disabled |
| 房间徽章 | `room-badge` | 显示房间名；点击回 rooms |
| 保存态 | `save-state` + `revision-display` | dirty / saving / saved / error |
| Presence | `room-presence` / `presence-online` | 在线成员头像 |
| 邀请 | `btn-invite` | 打开邀请模态；非 Owner/Editor disabled |
| 成员 | （图标按钮） | `open-drawer` → `room-members-panel` |
| 代码视图 | `btn-code-view` | SQL/DBML/JSON 只读生成 |
| 更多 | `btn-more-menu` | 导入 / 导出 / 分享 / 主题 / 命令等 |
| 用户菜单 | `user-menu` | 会话指示、偏好设置、退出 |

**移除为默认路径的项**：AppBar 常驻导入/导出 pill（改由更多菜单，见 `core-01d`）。

## ADDED — §1.2 成员抽屉 · 设置 / 分享模态

| UI | `data-testid` / 层 | 行为 |
|---|---|---|
| 成员抽屉 | `room-members-panel` | 在线数、角色变更、移除确认；邀请入口 |
| 邀请模态 | `modal-invite` / `invite-url` | 角色 Editor/Viewer；7 天有效文案 |
| 分享模态 | share 层 | 房间可编辑 vs 持链接只读；复制链接 |
| 偏好设置 | settings 层 | 主题、网格、自动保存等 |
| 创建房间 | `modal-create-room` | 位于 rooms 页，不在编辑器默认打开 |

## ADDED — §12 AppBar IO 按钮与 IO 抽屉（Phase C）

| 入口 | 行为 |
|------|------|
| `btn-more-menu` → `btn-import` | 打开 `import-drawer` |
| `btn-more-menu` → `btn-export` | 打开 `export-drawer` |
| `btn-more-menu` → `btn-share` | 打开分享模态 |
| Command Palette | 可命令打开同一 IO 抽屉 |

## ADDED — §1.3 响应式与 Viewer

- ≤1179：可隐藏 `room-badge`；保存 chip 可省略 revision 文案。
- ≤760：隐藏品牌分隔、save-chip、presence；操作区仅保留 `mobile-keep`（邀请、成员、更多等）。
- **Viewer**：undo/redo/标题/邀请写操作按 `canEdit` / `canInvite` disabled；仍可见 presence、代码视图、只读画布。

## ADDED — §8.x 边界补充

- ❌ 将主原型诊断面板、演示控制台标为生产必选
- ❌ 以独立 `core-03/04/05-*-prototype.html` 验收 AppBar
- ✅ AppBar 锚点：room-badge、save-state、presence、invite、code、more、user-menu
- ✅ 成员抽屉 + 设置/分享模态与主原型层叠一致
