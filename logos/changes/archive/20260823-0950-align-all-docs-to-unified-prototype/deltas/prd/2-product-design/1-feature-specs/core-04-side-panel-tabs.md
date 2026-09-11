# Delta — core-04-side-panel-tabs.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。侧栏语义以 **Tool Rail + Inspector + 抽屉** 为准，不再以 V1 左栏 7 Tab 为产品主路径。

| 项 | 约定 |
|---|---|
| 页面流 | `auth → rooms → room-editor`；侧栏工具仅在 room-editor |
| 演示 ≠ 生产 | 协作动态抽屉、演示角色切换仅为体验；生产以房间成员 API / WS 为准 |
| 实现状态 | **后端已实现**；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| Inspector | **`data-testid="inspector"`**（禁止 `inspector-panel`） |

## MODIFIED — 1. 侧边栏布局

V2 编辑器壳（与主原型 / IA 一致）：

| 区域 | `data-testid` | 职责 |
|---|---|---|
| Tool Rail | `tool-rail` | 建表、关系、区域、便签、命令搜索、协作动态、设置 |
| Canvas | `editor-canvas` | 对象交互；远端光标；连接 Banner |
| Inspector | `inspector` | 选中表/对象属性；可折叠 |
| 成员抽屉 | `room-members-panel` | 成员与角色 |
| 活动抽屉 | `activity-feed` | 协作动态（可选） |
| IO 抽屉 | `import-drawer` / `export-drawer` | 经更多菜单 |

**Tool Rail 主按钮（对齐主原型）**：

| 按钮 | testid | 说明 |
|---|---|---|
| 新建表 | `tool-add-table` | Viewer disabled |
| 关系 | `tool-relationship` | 进入关系工具；见 `core-01b` |
| 区域 / 便签 | — | 画布添加 |
| 搜索与命令 | `tool-search` | 打开 Command Palette |
| 协作动态 | — | `open-drawer` activity |
| 画布设置 | — | 打开设置模态 |

V1 §2～§7 各业务 Tab **仅作历史行为记录**；浏览/搜索迁至 Command Palette，属性编辑迁至 Inspector。

## ADDED — §1.x 响应式

- ≤1179：Inspector 叠在画布右侧，默认宽度约 330px；可关闭。
- ≤760：Tool Rail 改底栏横排；Inspector 绝对叠层；写工具保留 `mobile-keep` 可达性策略见 `core-05`。

## ADDED — §8.x Issues / 校验

Issues 不以左栏 Tab 为唯一入口；可与 StatusBar / 折叠条 / 命令面板并存。主原型未单独展示 Issues Tab 时，生产实现不得倒退回强制 280px 七 Tab 左栏。

## ADDED — §12.x Viewer 只读

- Tool Rail 写按钮 disabled。
- Inspector 输入与删除 disabled。
- 仍可打开成员抽屉查看（不可改他人角色，除非 Owner 管理规则另述）。
