# Delta — core-01c-index-enum-custom-type.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## MODIFIED — §1 索引（Index）— E3 Tag 视觉

**merge 时在 §1 末尾追加**：

### §1.x 索引徽章（E3 Tag）

| 索引类型 | E3 Tag color | 标签文本 | Icon |
|---|---|---|---|
| INDEX（普通） | `Neutral` | "IDX" | `<IconIndex />` |
| UNIQUE | `Success` | "UQ" | `<IconUnique />` |
| PRIMARY | `Warning` | "PK" | `<IconKey />` |
| FULLTEXT | `Info` | "FT" | `<IconString />` |
| SPATIAL | `Info` | "SP" | `<IconLink />` |

**视觉**：inline-flex 20px 高，gap 4px，Tag 不带 icon 文字（仅文字）或带 icon 文字混合

## MODIFIED — §2 枚举（Enum）— E3 Collapse

**merge 时在 §2 末尾追加**：

### §2.x 枚举折叠面板（E3 Collapse）

V1 枚举详情用模态（`cdb-modal-enum`）。E3 升级为 E3 `<Collapse>` 内嵌在 Inspector 抽屉内：

```rust
<Collapse lazy_render={true} bordered={CollapseBordered::Default}>
  <CollapsePanel header=view! { <Tag color=Primary>ENUM</Tag> <span>{name}</span> }>
    <For each=values key=|v| v.id children=|v| view! {
      <div class="cdb-py-2">
        <Tag color=Neutral>{v.name}</Tag>
      </div>
    } />
  </CollapsePanel>
</Collapse>
```

## MODIFIED — §3 自定义类型（CustomType）— E3 Tag

**merge 时在 §3 末尾追加**：

### §3.x 自定义类型徽章（E3 Tag）

| 类型 | E3 Tag color | Icon |
|---|---|---|
| 域（domain） | `Primary` | `<IconType />` |
| 复合类型（composite） | `Info` | `<IconString />` |
| 范围（range） | `Neutral` | `<IconInt />` |
