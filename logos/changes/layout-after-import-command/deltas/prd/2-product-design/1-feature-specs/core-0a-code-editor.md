# Delta — core-0a-code-editor.md（修改）

> 模块：core | 提案：layout-after-import-command
> 关联 issue：#23

## MODIFIED — §7 Command Palette 列表项

在既有 Tables / Relationships 等浏览项之外，**新增固定 Action**：

| 字段 | 值 |
|---|---|
| kind | `Action` |
| id | `action:layout` |
| label | `整理布局` |
| subtitle | 可选：`力导向 · 对齐 MCP` |
| data-testid | `palette-action-layout` |
| Enter / 点击 | 对当前 diagram 执行 `force_directed_layout` → store 落账 → 关闭 Palette |

> 算法与触发条件见 `core-01b-relationship.md` §4.5。
