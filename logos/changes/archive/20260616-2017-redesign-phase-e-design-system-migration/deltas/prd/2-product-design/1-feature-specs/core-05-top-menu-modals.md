# Delta — core-05-top-menu-modals.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 + E4 + E5 增量）

## MODIFIED — §1 AppBar 单行布局（E3 Button + Dropdown 视觉）

**merge 时替换** §1 段，更新为：

### §1 AppBar 单行布局（V2 — E3 Button / Dropdown / Tooltip）

V1 双行顶栏（菜单 + 工具栏）已在 **Phase A** 中合并为单行 AppBar（`redesign-phase-a-layout` 已合并）。V2 进一步用 E3 组件统一视觉：

```
+------------------------------------------------------------------+
| [Logo] [File▼] [Edit▼] [View▼] [Help▼]    [↶][↷]  [Title]  [Save]  [🌙] [Import] [Export] [Share] [Code] [⚙] |
+------------------------------------------------------------------+
```

| 元素 | E3 组件 | 视觉 |
|---|---|---|
| Logo | inline img | 32×32px，左 padding 12px |
| 菜单 4 个 | `<Dropdown trigger=Click position=BottomLeft>` | `<Button variant=Tertiary>` 触发；菜单项 `<DropdownItem icon=IconCaretDown />` |
| Undo / Redo | `<Button variant=Tertiary size=Small>` | `<IconUndo />` / `<IconRedo />` |
| Title | inline `<input>` | `--cdb-font-size-base`, `--cdb-color-text-0` |
| Save | `<Button variant=Primary size=Small>` | `<IconSave /> "保存"` |
| Theme toggle | `<Button variant=Tertiary size=Small>` | `<IconSun />`（浅色）/ `<IconMoon />`（暗色） — **E5 接线** |
| Import / Export | `<Button variant=Secondary size=Small>` | `<IconImport />` / `<IconExport />` |
| Share | `<Button variant=Secondary size=Small>` | `<IconShare />` |
| Code | `<Button variant=Secondary size=Small>` | `<IconCode />` — **E4 接线（btn-code-view）** |
| Settings | `<Button variant=Tertiary size=Small>` | `<IconSettings />` |

**Tooltip**：所有 AppBar 按钮在 hover 250ms 后显示 Tooltip（E3 §5），内容为"按钮名 + 快捷键"（如 "导入 (Ctrl+I)"）。

**E4 增量**：AppBar 末尾新增 `<Button data-testid="btn-code-view" on_click=toggle_code_view>`。点击切换 `ViewMode::Canvas | ViewMode::Code`。Code 模式时隐藏 Tool Rail、Inspector、IO 抽屉。

**E5 增量**：Theme toggle 按钮实现 `data-mode` 切换 + `localStorage` 持久化。详见 `core-0b-dark-mode.md`。

## MODIFIED — §2 菜单项（保留 V1 语义 + E3 Dropdown 视觉）

**merge 时在 §2 顶部插入**：

> V2 菜单项语义与 V1 §2.1–§2.4 一致，**视觉**改为 E3 `<DropdownItem icon=... active=... disabled=...>` 渲染。点击触发对应动作（New 模态 / Ctrl+K 焦点 / 等等）。**新增项**：
>
> - File → "命令面板…" → 打开 E4 CommandPalette（`Ctrl+K`）
> - View → "代码视图" → 切换 E4 ViewMode
> - View → "主题" 子菜单（Light / Dark / System）— **E5 接线**
>
> 详细 Dropdown 行为见 `core-09-core-components.md` §4。

## MODIFIED — §3 9 模态（E3 Modal 视觉统一）

**merge 时替换** §3 段，更新为：

### §3 9 模态（E3 Modal 组件统一）

| 模态 | E3 Modal width | E3 Button（footer） | 触发 | data-testid |
|---|---|---|---|---|
| New | `Medium` (640px) | Primary "创建" + Tertiary "取消" | File → New / Ctrl+N | `cdb-modal-new` |
| Open | `Small` (400px) | Primary "打开" | File → Open / Ctrl+O | `cdb-modal-open` |
| Import | `Large` (800px) | Primary "导入" + Tertiary "取消" | File → Import | `cdb-modal-import` |
| ImportSource | `Medium` (640px) | Primary "选择" | Import 模态内部 | `cdb-modal-import-source` |
| Language | `Small` (400px) | Primary "应用" | File → Settings → Language | `cdb-modal-language` |
| SetTableWidth | `Small` (400px) | Primary "应用" | View → Set Width | `cdb-modal-set-width` |
| Share | `Medium` (640px) | Primary "复制链接" | AppBar Share / Ctrl+Shift+S | `cdb-modal-share` |
| Rename | `Small` (400px) | Primary "重命名" | File → Rename | `cdb-modal-rename` |
| ConfigureCustomTypes | `Large` (800px) | Primary "保存" | View → Custom Types | `cdb-modal-custom-types` |

**E3 Modal 行为**（来自 `core-09-core-components.md` §3）：
- `centered=true`
- `esc_closable=true` / `mask_closable=true`
- 打开时 body 锁滚动
- focus trap：焦点循环在 modal 内
- 关闭时清空临时 state（`afterClose` 钩子，对齐 main `Modal.jsx`）

**遮罩生命周期约束**（V1 §4.1 保留）：
- `<div class="cdb-modal-overlay">` 仅在 `modal.get().is_some()` 时存在
- 模态关闭（modal 回到 None）时遮罩必须从 DOM 移除
- 失效时遮罩会持续拦截 pointer events，HP-01~HP-05 回归验收点

## MODIFIED — §4.2 模态布局（E3 Modal body style 对齐）

**merge 时替换** §4.2 段，更新为：

### §4.2 布局（E3 Modal body style 对齐 main）

```
+--------------------------------------+
| [Title]                    [×]       |   ← Modal header
+--------------------------------------+
|                                      |
|  body (maxHeight: viewport - 280)    |   ← Modal body
|  overflow: auto                      |
|                                      |
+--------------------------------------+
|                  [Cancel] [OK]       |   ← Modal footer
+--------------------------------------+
```

| 区域 | 视觉 | 来源 |
|---|---|---|
| Header | `--cdb-color-bg-0`, `--cdb-font-size-md`, `--cdb-font-weight-semibold` | E3 Modal |
| Body | `maxHeight: calc(100vh - 280px)`, `overflow: auto` | main `bodyStyle.maxHeight` |
| Footer | `padding: 12px 16px`, gap `12px`, right-aligned | E3 Modal 默认 |
| 关闭 × | `<IconClose />`, 32×32 圆形按钮, hover `--cdb-color-grey-1` | E3 Button Tertiary |

**Code/Image 模态特殊**：body `overflow: hidden`（避免 Monaco 滚动冲突），高度自适应内容。E4 Code View 用 `XLarge` (1200px)。
