## ADDED — §6.5 Inspector Tab 图标栅格（R5）

> 模块：core | 提案：r5-inspector-tabs

R5 将 Inspector 内 7 业务 Tab + **字段 Tab** 从文字换行栏改为 **4×2 图标栅格**：

```
+-- Inspector 320px ──────────────+
| [表][区][枚][注]               |
| [关][型][!][键]               |  ← icon-only + title tooltip
| [搜索........................]  |  ← 非字段 Tab 时显示
| ┌─ tab content（全高）───────┐ |
| │                            │ |
| └────────────────────────────┘ |
+--------------------------------+
```

| Tab | 图标 | testid | Tooltip |
|---|---|---|---|
| 表 | `IconAddTable` | `tab-tables` | 表 |
| 区域 | `IconAddArea` | `tab-areas` | 区域 |
| 枚举 | `IconEnum` | `tab-enums` | 枚举 |
| 注释 | `IconAddNote` | `tab-notes` | 注释 |
| 关系 | `IconRelationship` | `tab-relationships` | 关系 |
| 类型 | `IconType` | `tab-types` | 类型 |
| 问题 | `IconWarning` | `tab-issues` | 问题 |
| **字段** | `IconKey` | `tab-fields` | 字段 |

**字段 Tab（R5）**：

- 原 `.cdb-side-panel--right` 45% 底部分割 **废弃**；`field-editor` 仅在 `tab-fields` 激活时全高渲染
- 选中表/字段时自动切换至 `tab-fields`（对齐 S01 field-editor 可见性）
- 移除 `data-testid="right-panel"` 容器（`field-editor` testid 保留）

**移除项（R5）**：

- `.cdb-tabs--wrap` 文字换行 Tab 栏
- `.cdb-inspector > .cdb-side-panel--right { max-height: 45% }` 分割布局
