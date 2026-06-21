## ADDED — 15.2 AppBar 分区间距（R4）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-app-bar-gap` | `var(--cdb-space-3)` | AppBar 主 flex 分区间距（12px） |
| `--cdb-app-bar-brand-gap` | `var(--cdb-space-2)` | 品牌区内控件间距（8px） |
| `--cdb-app-bar-actions-gap` | `var(--cdb-space-2)` | 操作区按钮间距（8px） |

> R4 禁止 AppBar 使用非网格魔法数（如 `gap: 6px; padding: 4px` 的 IO pill 容器）；溢出菜单复用 `--cdb-z-popover` 层级。

## MODIFIED — 17. 验收约束

在现有 R3 约束后追加：

- AppBar 必须存在 `.cdb-app-bar__brand` / `__status` / `__actions` 三区 DOM
- 保存反馈必须通过单一 `.cdb-status-chip`（`data-testid="save-state"`）呈现
- `revision-display` **仅**出现在状态 Chip 内（StatusBar 不得重复）
- 导入/导出/主题通过 `btn-more-menu` 溢出菜单可达，testid 保持不变；Inspector 折叠仍用 StatusBar `btn-inspector-toggle`
