# Delta — core-05-top-menu-modals.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#13 / #14（顶栏「空间」返回按钮样式难看，两 issue 重复）、#15（顶栏协作内容被截断）

## MODIFIED — AppBar 统一工作空间信息分层补充

```
[返回] btn-back-to-rooms
[品牌] Logo · Undo/Redo · diagram-title · room-badge · save-state
                                    …spacer（弹性，min-width:0）…
[操作] presence · invite · 成员 · code · more · user-menu
```

| 元素 | `data-testid` | 说明 |
|---|---|---|
| 顶栏容器 | `app-bar` | 玻璃态 AppBar |
| 返回空间 | `btn-back-to-rooms` | 「← 空间」横排单一入口；点击返回空间/房间列表（功能不变） |
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

### 返回空间入口样式合同（#13/#14，fix-remote-github-issues-7-18）

> 根因：生产 `btn-back-to-rooms` 使用 `cdb-btn--ghost` 但保留了可见边框，且 `.cdb-btn` 无 `inline-flex` 居中，与主原型 ghost 语言（`.btn--ghost{border-color:transparent}` + inline-flex）不一致。

| ID | 约束 |
|---|---|
| R-AB-01 | 返回入口为**单一横排按钮**：图标 + 「空间」文字同行（`inline-flex` + `align-items:center` + 统一 gap），禁止图标与文字纵向堆叠或错位 |
| R-AB-02 | ghost 语义对齐主原型：透明背景、**透明边框**（`border-color: transparent`），hover 才显现 `--cdb-color-surface-hover` 底色；不得常驻可见边框 |
| R-AB-03 | hover / active / focus-visible 态明确（focus 用 `--cdb-color-focus-ring`）；暗色模式对比度达标 |
| R-AB-04 | 可点击区域高度 ≥ 36px（与 AppBar 其他控件同档）；窄屏不挤压标题区 |
| R-AB-05 | 可访问性：`title="返回空间列表"` 与可识别文本/aria-label 保持；点击行为不变（返回空间/房间列表） |

### 协作芯片与中部截断策略（#15，fix-remote-github-issues-7-18）

> 根因：`.cdb-room-badge` 的 `text-overflow: ellipsis` 写在容器 `<button>` 上而文本在内部 `<strong>`（无截断规则），且 AppBar 中部无弹性收缩约定。

| ID | 约束 |
|---|---|
| R-AB-06 | 文本截断规则必须落在**文本节点**上：`room-badge` 的 `<strong>` 与 `diagram-title` 各自具备 `min-width:0` + `overflow:hidden` + `text-overflow:ellipsis` + `white-space:nowrap`（对齐主原型 `.room-badge strong` 口径） |
| R-AB-07 | 优先完整展示；空间不足时省略号截断，且 hover / Tooltip（`title`）必须显示全文（房间名、图名同此） |
| R-AB-08 | 中部区（标题 + room-badge + save-state）建立弹性收缩策略：容器 `min-width:0`、spacer 弹性挤压先行；`save-state` 与操作区 `flex:0 0 auto` 不被压缩；长房间名不得把「+ 邀请」挤出视口 |
| R-AB-09 | 响应式保持既有口径（§1.3：≤1179 可隐藏 `room-badge`、保存 chip 可省略 revision 文案）；截断场景一律有 `title` 全文兜底 |

### 测试锚点（新增）

| TC ID | 描述 |
|---|---|
| UT-PE-AB-01 | 样式锚点：`.cdb-btn--ghost` 透明边框 + `.cdb-back-to-rooms` inline-flex 居中（`styles.css` 断言） |
| UT-PE-AB-02 | 样式锚点：`.cdb-room-badge strong` 携带 ellipsis 三件套；badge 容器有 `max-width` 与 `title` 全文 |
| ST-PE-07 | e2e：长房间名（≥24 字符）下 room-badge 省略截断且 title 全文可见；「+ 邀请」可见可点 |

> 详细步骤见 `core-PE-design-system-test-cases.md`。
