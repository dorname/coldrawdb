## ADDED — 5. IconBox 尺寸容器（R1）

`frontend-rs/src/icons.rs` 提供统一包装组件，避免在 UI 层硬编码像素：

```rust
#[component]
pub fn IconBox(
    #[prop(default = "sm")] size: &'static str,  // "sm" | "md" | "lg"
    children: Children,
) -> impl IntoView
```

| size prop | CSS 类 | 场景 |
|---|---|---|
| `"sm"` | `.cdb-icon-wrap--sm` (16px) | `.cdb-btn--icon`、Drawer 标题、Modal 关闭 |
| `"md"` | `.cdb-icon-wrap--md` (20px) | `.cdb-tool-btn` |
| `"lg"` | `.cdb-icon-wrap--lg` (24px) | EmptyGuide 装饰 |

## ADDED — 6. R1 新增图标（画布工具）

| 图标 | 用途 | 替换原占位 |
|---|---|---|
| `IconSelect` | 选择工具 | `↖` |
| `IconSidebar` | Inspector 侧栏切换 | `☰` |

## MODIFIED — 4.4 画布对象（5 个，main 自建）

| 图标 | 用途 | 替换原 emoji | 接线位置 |
|---|---|---|---|
| `IconSelect` | 选择工具 | `↖` | ToolRail `tool-select` |
| `IconAdd` | 新建菜单 | `⊕` | ToolRail `tool-new-menu` |
| `IconRelationship` | 关系工具 | `🔗` | ToolRail `tool-relationship` |
| `IconPan` | 平移工具 | `✋` | ToolRail `tool-pan` |
| `IconAddTable` | 新建表（菜单项，可选） | — | tool-new-menu dropdown |

> **R1 验收**：`editor_panels.rs` 中 ToolRail / AppBar / StatusBar / IO Drawer 不得再使用 Emoji/Unicode 作为图标占位；Logo 字母 `C` 保留为品牌字标。
