# Delta — core-00-information-architecture.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：信息架构（V1）

# 信息架构（V1 编辑器 + V2 工作空间）

## ADDED — 0. 现行页面状态（产品主路径）

唯一现行主原型：`core-01-editor-prototype.html`。生产前端与规格必须以以下页面状态为准：

| 状态 ID | 视图 | `data-testid` | 前置 | 下一跳 |
|---|---|---|---|---|
| `auth` | 登录 / 注册 | `login-form` / `register-form` | 未登录 | `rooms` |
| `rooms` | 房间与最近项目 | `rooms-list-page` | 已登录 | `room-editor` / `invite` |
| `invite` | 邀请预览 / 失效 | `invite-accept-page` | 邀请 token | `room-editor` 或停留失效 |
| `room-editor` | 协作 ER 编辑器 | `room-editor-page` | 房间成员 | 可回 `rooms` |
| `share-readonly` | 匿名只读分享 | （S02 画布只读） | `?share=` | 不强制登录 |

```text
auth ──登录成功──→ rooms ──创建/打开/接受邀请──→ room-editor
                      ▲                              │
                      └──────── room-badge / 退出 ────┘
?share= ──────────→ share-readonly（旁路，不被鉴权阻断）
```

历史 Landing → 空白 `/editor` 不再作为默认主路径。

## MODIFIED — 2. 路由

| 路由 / 状态 | 页面 | 实现要求 | 备注 |
|---|---|---|---|
| `/login` · `/register`（或同壳 `auth`） | 鉴权 | S03；已登录应进入 rooms | 主原型同壳切换，生产可用真实路由 |
| `/rooms` | 房间列表 | S04；需登录 | `rooms-list-page` |
| `/invite/:token` | 邀请 | S04；需登录后接受 | 过期无加入按钮 |
| `/rooms/:id/editor` 或等价 room-editor | 协作编辑器 | S01+S04+S05 | `room-editor-page` |
| `/?share=<id>` | 匿名只读 | S02 | 不被鉴权拦截 |
| `/templates` 等 | — | 仍不做 | 与既有 Out of Scope 一致 |

## MODIFIED — 10.6 原型与生产边界

### 10.6 原型与生产边界

主原型用于验证信息架构、状态反馈和交互连贯性。界面中的登录、HTTP、WebSocket、OT transform、剪贴板和文件下载可由本地模拟器驱动；所有模拟入口必须标注「演示」或「模拟」。

**状态表述约定**：

- 不得因主原型可演示而将 S03～S05 标为全栈完成。
- 当前准确表述：后端已实现；生产前端 API/页面流已部分接入；相对主原型的结构/视觉/交互逐项对齐，以本提案规格为合同，由下一变更 `implement-unified-prototype-spec-parity` 实现与验收。
- `core-03/04/05-*-prototype.html` 仅历史参考，不作为验收入口。

## ADDED — 编辑器壳层级与锚点（与主原型一致）

| 区域 | `data-testid` | 说明 |
|---|---|---|
| AppBar | `app-bar` | 撤销/重做、标题、房间徽章、保存态、presence、邀请、代码、更多、用户菜单 |
| ToolRail | `tool-rail` | 建表、关系、搜索/命令等 |
| Canvas | `editor-canvas` | 表/关系/区域/便签/远端光标/连接 Banner |
| Inspector | `inspector` | 选中对象属性；可折叠 |
| StatusBar | `status-bar` | `ws-status`、`ot-rev`、缩放、角色 |
| 成员抽屉 | `room-members-panel` | 成员与角色 |
| IO 抽屉 | 导入/导出入口经更多菜单 | 见 IO 规格 |
| Command Palette | `command-palette` | ⌘K / Ctrl+K |
| Code View | `code-view-modal` | SQL/DBML/JSON |

层级 L0～L6 玻璃态规则以既有 §10.4 为准；响应式三档以 §10.5 为准。
