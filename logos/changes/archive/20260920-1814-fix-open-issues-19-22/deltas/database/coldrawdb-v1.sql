# Delta — coldrawdb-v1.sql（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#21

## MODIFIED — `reference` 表

在 `color` 列定义后追加：

```sql
    -- fix-open-issues-19-22（#21）：线条类型 / 线型；缺省 bezier + solid
    -- 既有库迁移：backend/migrations/0010_reference_line_style.up.sql
    line_type           TEXT    NOT NULL DEFAULT 'bezier'
                        CHECK(line_type IN ('bezier', 'orthogonal', 'straight')),
    stroke_style        TEXT    NOT NULL DEFAULT 'solid'
                        CHECK(stroke_style IN ('solid', 'dashed')),
```

并修正既有注释中错误的迁移编号引用（若仍写 `0006_reference_color`）：应指向 `0009_reference_color`。

## 迁移文件（代码阶段产出，规格锚点）

- `backend/migrations/0010_reference_line_style.up.sql`：`ALTER TABLE reference ADD COLUMN line_type ...`；`ADD COLUMN stroke_style ...`
- `backend/migrations/0010_reference_line_style.down.sql`：`DROP COLUMN` 配对
