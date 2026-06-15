# Delta — core-01e-command-palette.md（新文件）

> merge 时作为新文件写入 `logos/resources/prd/2-product-design/1-feature-specs/core-01e-command-palette.md`

## ADDED — 全文

> 模块：core | 提案：redesign-phase-d-command-code
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-01e-command-palette.md`
> 对齐：drawdb Command Palette、`core-04-side-panel-tabs.md`（浏览能力迁移）
> 最后更新：2026-06-14

# Command Palette 规格（V2 / Phase D）

## 1. 概述

Phase D 以 **Command Palette** 恢复 Phase A 移除的左栏 7 Tab **浏览与跳转**能力，不恢复 280px 侧栏 UI。

- 触发：`Ctrl+K`（Windows/Linux）/ `Cmd+K`（macOS）
- 可选入口：AppBar View 菜单「命令面板…」
- 层级：L4.5 居中浮层（高于 Inspector / IO 抽屉，低于阻塞模态）

## 2. 组件树

```
AppRoot
├── ...
├── CommandPalette          ← data-testid="command-palette"
│   ├── command-palette-input
│   ├── command-palette-results
│   └── command-palette-item-{kind}-{id}
└── （全局 keydown Ctrl+K 监听）
```

## 3. 状态模型

```rust
enum PaletteKind {
    Table,
    Area,
    Enum,
    Note,
    Reference,
    CustomType,
    Action,
}

struct PaletteItem {
    kind: PaletteKind,
    id: String,
    label: String,
    subtitle: Option<String>,
}

// AppRoot 信号
palette_open: RwSignal<bool>
palette_query: RwSignal<String>
palette_highlight: RwSignal<usize>  // 键盘上下导航
```

## 4. 布局

```
┌─ 搜索命令或对象 ─────────────────────────────┐
│ 🔍 [ tables________________________ ]       │  ← command-palette-input
├──────────────────────────────────────────────┤
│ 表                                           │
│   users                          3 fields    │  ← command-palette-item-table-users
│   orders                         5 fields    │
│ 区域                                         │
│   billing                                    │
│ 操作                                         │
│   + 创建新表                                 │  ← command-palette-item-action-create-table
│   ↑ 打开导入抽屉                             │
└──────────────────────────────────────────────┘
```

- 宽度：**560px**，最大高度 **60vh**，垂直居中
- 遮罩：半透明 `rgba(0,0,0,.4)`，点击遮罩关闭
- testid：`command-palette` / `command-palette-backdrop` / `command-palette-input` / `command-palette-results` / `command-palette-empty`

## 5. 数据源与过滤

### 5.1 构建列表（纯函数 `build_palette_items(store) -> Vec<PaletteItem>`）

| kind | 来源 | label | subtitle |
|------|------|-------|----------|
| Table | `store.tables` | 表名 | `"{n} fields"` |
| Area | `store.areas` | 区域名 | 表数（可选） |
| Enum | `store.enums` | 枚举名 | 值数量 |
| Note | `store.notes` | 便签摘要（前 32 字符） | — |
| Reference | `store.references` | `{from_table}.{from_field} → {to_table}.{to_field}` | — |
| CustomType | `store.custom_types` | 类型名 | — |
| Action | 固定 | 见 §5.3 | — |

### 5.2 模糊过滤（`filter_palette_items(items, query)`）

- 大小写不敏感子串匹配 `label` + `subtitle`
- query 为空：显示全部（Action 组置顶或置底，V1 置底）
- 无匹配：显示空态「无结果」

### 5.3 固定操作项（Action）

| id | label | 行为 |
|----|-------|------|
| `action-create-table` | + 创建新表 | 关闭 Palette → 调用 `on_create_table` |
| `action-open-import` | ↑ 打开导入抽屉 | 关闭 Palette → `io_drawer = Import` |

## 6. 交互

| 操作 | 行为 |
|------|------|
| `Ctrl+K` / `Cmd+K` | 切换 `palette_open` |
| Esc | 关闭 Palette |
| ↑ / ↓ | `palette_highlight` 循环 |
| Enter | 执行当前高亮项 |
| 单击项 | 同 Enter |
| 跳转 Table | `selection = Table(id)`，`inspector_open = true`，关闭 Palette |
| 跳转 Reference | `selection = Reference(id)`，`inspector_open = true` |
| 跳转 Area / Note | 画布居中该对象（V1：选中 + inspector 若可） |

复用现有 `on_jump_to_table` / `jump_to_reference` 纯函数。

## 7. 互斥规则

| 事件 | 行为 |
|------|------|
| 打开 Palette | 关闭 IO 抽屉；不自动折叠 Inspector（跳转时才展开） |
| 阻塞模态已开（New/Share/冲突） | `Ctrl+K` 不响应 |
| Code 视图激活 | Palette 仍可用（V1 允许） |

## 8. 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-PD-01 | `build_palette_items` 含表与 Action |
| UT-PD-02 | `filter_palette_items("user")` 过滤 |
| UT-PD-03 | `is_palette_shortcut` Ctrl+K 检测 |
| UT-PD-07 | Palette 结果含 6 类对象（替代 UT-SP-09） |
| UT-PD-08 | 全局 query 跨类过滤（替代 UT-SP-10） |
| ST-PD-01 | e2e：Ctrl+K → 搜表名 → Enter → Inspector 显示表名 |

## 9. V1 边界

- ❌ 命令历史 / 最近使用
- ❌ 模糊拼音搜索
- ❌ 批量操作命令
- ❌ Palette 内嵌 SQL 编辑
