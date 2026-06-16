# Delta — core-01d-import-export.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## MODIFIED — §2 组件树（E3 SideSheet 升级）

**merge 时在 §2 末尾追加**：

### §2.x E3 SideSheet 重构

V1 IO 抽屉用内嵌 `<aside class="cdb-io-drawer">` 自实现。E3 升级为 `<SideSheet placement=Right width=400>` 组件（来自 `core-09-core-components.md` §9）：

```rust
<SideSheet
    visible=io_drawer_open
    title=move || format!("{kind:?}")
    placement=SideSheetPlacement::Right
    width=400
    mask={true}
    mask_closable={true}
>
    <ImportExportContent kind=kind />
</SideSheet>
```

**Props 差异**（V1 → E3）：
- `cdb-is-io-drawer-open` class → `visible: RwSignal<bool>` prop
- `cdb-io-drawer__close` 内嵌按钮 → `<Button variant=Tertiary icon=IconClose />` in header
- 关闭动画：手动 CSS → E3 内置 `slide-in-right` / `slide-out-right`（E6 接入）

## MODIFIED — §4–§5 ImportDrawer / ExportDrawer（E2 复制/下载图标）

**merge 时在 §4、§5 各追加**：

### §4.x / §5.x 抽屉内操作按钮（E2 + E3）

| 行为 | 组件 | 视觉 |
|---|---|---|
| 复制导入源 / 复制导出结果 | `<Button variant=Secondary icon=IconCopy>复制</Button>` | E3 Button Secondary |
| 下载导出文件 | `<Button variant=Primary icon=IconDownload>下载</Button>` | E3 Button Primary |
| 拖入文件 | `<div class="cdb-dropzone"><IconUpload />"拖入文件或点击选择"</div>` | E2 Icon + E3 Collapse-style border |
| 切换数据库（MySQL/PostgreSQL/SQLite/...） | `<Dropdown trigger=Click position=BottomLeft>` | E3 Dropdown |

**ImportDrawer** 头部增加 `<Tag color=Info size=Small>SQL/DBML/JSON</Tag>` 标识当前 format。
