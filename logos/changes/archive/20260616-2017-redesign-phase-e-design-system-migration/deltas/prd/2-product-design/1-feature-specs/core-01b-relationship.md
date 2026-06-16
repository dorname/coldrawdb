# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## MODIFIED — §3 关系操作（E3 Tooltip / Popover 替换）

**merge 时在 §3 末尾追加**：

### §3.x 关系工具 Tooltip / Popover（E3）

关系工具激活时（`cdb-tool-rail-relationship` 按钮）显示：

| 元素 | E3 组件 | 内容 |
|---|---|---|
| 起点表 hover | `<Tooltip placement=Top>` | `"{table_name} ({field_count} 字段)"` |
| 起点字段 hover | `<Tooltip placement=Top>` | `"{field_name} : {field_type}"` |
| 终点表 hover | `<Tooltip placement=Top>` | `"{table_name}"` |
| 关系线 hover | `<Popover trigger=Click>` | 关系详情：起点、终点、cardinality、onUpdate / onDelete（来自 main `RelationshipInfo.jsx`） |
| 关系线点击 | `<Popover>` 展开 | 同上 |

**视觉**：
- Tooltip：黑底白字，`--cdb-shadow-md`，`--cdb-radius-sm`
- Popover：白底，`--cdb-shadow-md`，`--cdb-radius-lg`，内容宽度 280px

**z-index**：Tooltip `--cdb-z-tooltip`（L4），Popover `--cdb-z-popover`（L4.5）

## MODIFIED — §4 渲染（E2 关系线端点图标）

**merge 时在 §4 末尾追加**：

### §4.x 关系线端点图标（E2）

- 起点端点：`<IconKey />` 或 `<IconLink />`（小尺寸 12px）
- 终点端点：`<IconCaretDown />` 旋转 90°（one-to-many 视觉）
- 颜色：`var(--cdb-color-primary)`（默认）、`var(--cdb-color-warning)`（hover）
- 选中态：线粗 2.5px，色 `--cdb-color-primary-active`
