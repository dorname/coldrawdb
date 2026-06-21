## ADDED — §1.1 AppBar 信息分层（R4）

> 模块：core | 提案：r4-appbar-hierarchy

R4 将 Phase A/E 单行 AppBar 进一步划分为 **三区**，降低 48px 行内控件密度：

```
+--------------------------------------------------------------------------------+
| [品牌区]  Logo · Undo/Redo · 标题          | [状态 Chip]  ● 已保存 · rev:5  |
|                              [操作区]  [分享] [代码] [⋯]                        |
+--------------------------------------------------------------------------------+
```

| 分区 | class | 内容 | testid |
|---|---|---|---|
| 品牌区 | `.cdb-app-bar__brand` | Logo + Undo/Redo + `diagram-title` | `diagram-title` / `btn-undo` / `btn-redo` |
| 状态区 | `.cdb-app-bar__status` | 单一 `.cdb-status-chip` | `save-state` + 内嵌 `revision-display` |
| 操作区 | `.cdb-app-bar__actions` | Share Primary + Code + Overflow | `btn-share` / `btn-code-view` / `btn-more-menu` |

**状态 Chip（R4）**：

- 合并原 `save-state` + `revision-display` + 标题 dirty 点；**禁止**在标题旁单独渲染 dirty 圆点
- 结构：`[圆点] [状态文案] · rev: N`（rev 使用 `data-testid="revision-display"`）
- 四种态与 R3 语义色一致：已保存 / 未保存 / 保存中 / 离线失败

**溢出菜单 `⋯`（R4）**：

| 菜单项 | testid | 说明 |
|---|---|---|
| 导入 | `btn-import` | 打开 IO 抽屉 Import 面板 |
| 导出 | `btn-export` | 打开 IO 抽屉 Export 面板 |
| 切换主题 | `btn-theme-toggle` | Light ↔ Dark（与 E5 行为一致） |

> Inspector 折叠仍由 **StatusBar** `btn-inspector-toggle` 承载（R4 仅移除 AppBar 末尾重复图标）。

**移除项（R4）**：

- AppBar 内 `.cdb-app-bar__actions` IO pill 容器（导入/导出文字按钮）
- AppBar 末尾独立 `btn-theme-toggle` / `btn-inspector-toggle` 图标按钮
- StatusBar 内重复的 `revision-display`（rev 仅由状态 Chip 承载）

## MODIFIED — §1 AppBar 单行布局（E3 Button + Dropdown 视觉）

**merge 时替换** §1 段首布局示意与元素表，更新为：

### §1 AppBar 单行布局（V2 — R4 信息分层）

V1 双行顶栏已在 Phase A 合并；E3 统一按钮视觉；**R4** 进一步三区 + 溢出菜单（见 §1.1）。

| 元素 | E3 组件 | 分区 | testid |
|---|---|---|---|
| Logo | inline | 品牌区 | — |
| Undo / Redo | Tertiary icon | 品牌区 | `btn-undo` / `btn-redo` |
| Title | inline input | 品牌区 | `diagram-title` |
| Save Chip | Tag-like chip | 状态区 | `save-state` + `revision-display` |
| Share | Primary Small | 操作区 | `btn-share` |
| Code | Tertiary icon | 操作区 | `btn-code-view` |
| More | Tertiary icon + Dropdown | 操作区 | `btn-more-menu` → 内含 import/export/theme |

**Tooltip**：操作区按钮 hover 250ms 显示 Tooltip（E3 §5）。

**E4 / E5**：Code View 与 Theme 行为不变；Theme 入口自 AppBar 图标迁至溢出菜单（`btn-theme-toggle` testid 保留在菜单项）。
