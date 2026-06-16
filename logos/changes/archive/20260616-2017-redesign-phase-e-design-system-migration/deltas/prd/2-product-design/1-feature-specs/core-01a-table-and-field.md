# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E2 增量）

## MODIFIED — §2 字段（Field）— 字段类型徽章 E2 Tag + Icon

**merge 时在 §2 末尾追加**：

### §2.x 字段类型徽章（E2 Tag + Icon）

V1 字段类型用 `text-orange-500`（main `stringColor`）等 Tailwind 颜色 + emoji 表示。E2 升级为 E3 `<Tag color=... size=Small>` + E2 Icon：

| 字段类型 | E2 图标 | E3 Tag color | 颜色 token |
|---|---|---|---|
| string / varchar / text | `<IconString />` | `Primary` | `--cdb-color-primary-soft` |
| int / integer / bigint | `<IconInt />` | `Warning` | `--cdb-color-warning-soft` |
| decimal / numeric / float | `<IconDecimal />` | `Success` | `--cdb-color-success-soft` |
| boolean / bool | `<IconBoolean />` | `Info` | `--cdb-color-info-soft` |
| date / datetime / timestamp | `<IconDate />` | `Info` | `--cdb-color-info-soft` |
| enum (drawdb enum) | `<IconEnum />` | `Primary` | `--cdb-color-primary-soft` |
| binary / blob | `<IconBinary />` | `Success` | `--cdb-color-success-soft` |

**Props 签名**：
```rust
<FieldTypeBadge type_: FieldType, size: TagSize = TagSize::Small />
```

**视觉**：inline-flex 22px 高，gap 4px，Icon size=12，Tag 不带文字（仅 icon + soft 背景）

## MODIFIED — §2 主键/外键图标（E2 Icon 替换）

**merge 时在 §2 主键/外键说明处替换**：

- 主键：`<IconKey />` + `<Tag color=Warning size=Small>PK</Tag>`（`--cdb-color-warning` 文字色）
- 外键：`<IconLink />` + `<Tag color=Info size=Small>FK</Tag>`
- 索引：`<IconIndex />` + `<Tag color=Neutral size=Small>IDX</Tag>`
- 唯一约束：`<IconUnique />` + `<Tag color=Success size=Small>UQ</Tag>`
- 非空：`<IconNotNull />`（无 Tag，纯图标 hover 提示）
