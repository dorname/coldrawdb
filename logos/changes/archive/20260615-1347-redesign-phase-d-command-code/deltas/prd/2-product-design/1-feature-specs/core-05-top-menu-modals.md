# Delta — core-05-top-menu-modals.md

## ADDED — §13 AppBar 代码视图与 Command Palette（Phase D）

> merge 时在 §12 之后追加。

### 13.1 AppBar 新增 / 启用

| 按钮 | testid | Phase D 行为 |
|------|--------|--------------|
| 代码 | `btn-code-view` | **启用**；切换 `EditorViewMode::Code` |
| （无新按钮） | — | Palette 仅快捷键 + View 菜单 |

### 13.2 View 菜单

| 菜单项 | testid | 行为 |
|--------|--------|------|
| 代码视图 | `cdb-menu-code-view` | 同 `btn-code-view` |
| 命令面板… | `cdb-menu-command-palette` | 打开 Command Palette |

### 13.3 快捷键（全局）

| 快捷键 | 行为 |
|--------|------|
| `Ctrl+K` / `Cmd+K` | 切换 Command Palette |
| `Esc`（Code 模式） | 返回 Canvas |

### 13.4 Phase D 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-PD-03 | `is_palette_shortcut` |
| UT-PD-06 | `btn-code-view` enabled |

## MODIFIED — §1 顶栏布局（补充）

> AppBar 按钮序（Phase D）：`… | 导入 | 导出 | 代码 | ↶↷ | 分享 | …`
