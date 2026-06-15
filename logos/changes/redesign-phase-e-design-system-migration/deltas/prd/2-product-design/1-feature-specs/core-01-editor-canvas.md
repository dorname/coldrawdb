# Delta — core-01-editor-canvas.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 + E4 增量）

## MODIFIED — §5.2 布局栅格（E1 token 引用统一）

**merge 时替换** §5.2 段，更新为：

### §5.2 布局栅格（V2 — E1 token 引用）

> 完整栅格定义见 `core-00-information-architecture.md` §1 + `core-04-side-panel-tabs.md` §1（Tool Rail）。

| CSS variable | 值 | 来源 |
|---|---|---|
| `var(--cdb-color-bg-0)` | `#ffffff` | 主背景 |
| `var(--cdb-color-bg-3)` | `#e5e7eb` | 画布背景（bg-canvas） |
| `var(--cdb-color-text-0)` | `#1f2937` | 主要文字 |
| `var(--cdb-color-text-2)` | `#6b7280` | 辅助文字 |
| `var(--cdb-color-border)` | `#e5e7eb` | 边框 |
| `var(--cdb-color-primary)` | `#175e7a` | AppBar / 链接 / 聚焦 |
| `var(--cdb-shadow-sm)` | — | AppBar / Card |
| `var(--cdb-radius-md)` | `6px` | 按钮 / 输入框 |
| `var(--cdb-z-app-bar)` | `20` | AppBar / StatusBar |
| `var(--cdb-z-side-rail)` | `25` | Tool Rail |
| `var(--cdb-z-inspector)` | `30` | Inspector 抽屉 |
| `var(--cdb-z-drawer)` | `30` | IO 抽屉 |
| `var(--cdb-z-popover)` | `45` | Dropdown / Popover |
| `var(--cdb-z-modal)` | `50` | Modal / CommandPalette |

**E1 增量**：所有硬编码颜色/阴影/圆角值替换为 `var(--cdb-*)` 引用。`styles.css` 顶部扩展 ~100 token，移除临时硬编码。

## MODIFIED — §6 与侧栏 / 顶部菜单的联动（E3 Inspector 组件对齐）

**merge 时在 §6 末尾追加**：

### §6.4 Inspector 组件（E3 重构）

Inspector 抽屉（L3）承载画布选中态的属性编辑。E3 阶段用 E3 组件重构：

| 区域 | E3 组件 | 视觉 |
|---|---|---|
| Header | `<Tag color=Primary>{kind}</Tag>` + `<h3>{name}</h3>` | `--cdb-color-bg-1` |
| 属性编辑 | `<Input>` / `<Textarea>` / `<Select>` (V1 自实现 → E3 暂用 `cdb-input`) | `--cdb-color-bg-0` |
| 操作按钮 | `<Button variant=Primary>保存</Button>` + `<Button variant=Warning>删除</Button>` | 见 `core-09-core-components.md` §2 |
| 关闭 | `<Button variant=Tertiary icon=IconClose />` | — |

**E4 增量**：Inspector 头部增加"Code View"入口（`<Button on_click=open_code_view_for_selected>`），仅在选中表/关系时显示。

## MODIFIED — §9 详细规格（E3 验收更新）

**merge 时在 §9 末尾追加**：

### §9.x E3 验收约束

- 画布所有交互元素可被 Tooltip（E3 §5）标注
- 选中态用 `--cdb-color-primary-soft` 高亮
- 拖拽用 `--cdb-cursor-grab` / `--cdb-cursor-grabbing`
- 画布背景：浅色 `--cdb-color-bg-3`，暗色（E5）`--cdb-color-bg-2`
