# Delta: core-01a-table-and-field.md

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题3：数据库导入注释丢失 + 注释展示）

背景：`Table.comment`（§1.1）与 `Field.comment`（§2.1）数据模型早已存在，JSON 导入已透传 comment；但**表注释在 UI 无任何展示/编辑入口**，字段注释仅 ListView 字段网格有编辑列。本 delta 补齐表注释展示与编辑，并声明 Inspector 字段卡注释输入。

## MODIFIED — §2.3 字段操作

字段增删改、类型与约束的主编辑面为右侧 **Inspector**（选中表后展开），与主原型 `renderInspector` 一致：

- 表名 / 表注释（见 §1.4）/ 强调色
- 字段列表卡片：名称、类型、PK / NOT NULL / UNIQUE、**注释输入**（见下）、删除
- 「添加」字段、删除数据表（确认模态）

**字段注释编辑入口（双轨）**：

| 位置 | 形态 | testid | 说明 |
|---|---|---|---|
| ListView 字段网格第 2 列 | 行内注释输入（**已存在**，`editor_panels.rs` `list-field-comment-{fid}`） | `list-field-comment-{field_id}` | 受控输入，blur 落账；本 delta 不改动其行为 |
| Inspector 字段卡 | 每张字段卡片追加注释输入（**新增**） | `inspector-field-comment-{field_id}` | 受控输入，blur 落账通路仿 `on_rename`（写 store → undo → schedule_save） |

- 字段注释不参与 §2.4 校验（任意字符串，允许为空）
- Viewer（`canEdit() === false`）与 `locked === true` 时两处输入均 disabled
- 字段注释变更进入 `UndoRedoContext`（§4）

## ADDED — §1.4 表注释展示与编辑

> 提案：fix-canvas-zoom-invite-comment-resize

`Table.comment` 为 SQL 表注释（`COMMENT ON TABLE ...` / MySQL 表选项）。V1 仅有数据字段、无 UI 入口；本提案补齐展示与编辑。

### 1.4.1 ListView 表树展示（只读）

- 表树节点（`list-tree-node-{表名}`，现状仅表名 + `N 字段` 计数）在表名下方追加注释行：
  - 有注释：`<span class="cdb-list-tree-node__comment" data-testid="list-table-comment-{table_id}">{comment}</span>`，单行截断（`text-overflow: ellipsis`），最大宽度与节点同宽
  - **无注释：不渲染该节点、不占位**，节点高度与现状一致
- 纯展示，点击行为不变（单击选中 / 双击跳转画布）

### 1.4.2 Inspector 数据表 section 注释编辑（新增）

Inspector 顶部「数据表」section（`cdb-panel-section`，现有表名输入 `inspector-table-name` 与强调色）在表名输入下方追加注释输入：

```rust
<div class="cdb-form-group">
    <label>"注释"</label>
    <input
        class="cdb-form-input"
        data-testid="inspector-table-comment"
        prop:value=table_comment            // 受控：来自当前选中表 Table.comment
        disabled=ro                         // Viewer / locked 同 inspector-table-name
        on:blur=move |ev| {
            if !ro {
                on_set_table_comment(table_id, event_target_value(&ev));
            }
        }
    />
</div>
```

- **落账通路仿 `on_rename_table`**：blur 时若值变化 → 写 store（`Table.comment`）→ 进 `UndoRedoContext`（§4）→ `dirty` + `schedule_save` PUT 当前 diagram
- 表注释不参与 §1.3 校验（任意字符串，允许为空；不校验 SQL 标识符规则）
- 注释变更**不改表名**、不触发重名检查

### 1.4.3 与导入/导出的关系

- 数据库导入（`core-01d` §4.4 第 3.5 条）与 JSON 导入透传的表/列注释落入同一 `Table.comment` / `Field.comment`，展示与编辑入口即时可见
- 导出（`core-01d` §5.2）在表注释非空时输出 `COMMENT ON TABLE`（PG/generic）或 MySQL 表选项 `COMMENT='...'`；空注释不输出

### 1.4.4 testid 汇总

| testid | 元素 | 交互 |
|---|---|---|
| `list-table-comment-{table_id}` | 表树注释行 | 只读 |
| `inspector-table-comment` | Inspector 表注释输入 | blur 落账 |
| `inspector-field-comment-{field_id}` | Inspector 字段卡注释输入 | blur 落账 |
| `list-field-comment-{field_id}` | ListView 字段网格注释输入（已存在） | blur 落账 |

对应测试用例见 `core-PC-import-export-test-cases.md` UT-PC-29 / UT-PC-30 / ST-PC-08。
