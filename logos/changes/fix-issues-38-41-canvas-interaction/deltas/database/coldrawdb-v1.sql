# Delta — coldrawdb-v1.sql（修改）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#41

## MODIFIED — `area` 表

在 `name` 列定义后追加：

```sql
    -- fix-issues-38-41（#41）：区域锁定；0 = 未锁定（缺省，旧图兼容），1 = 锁定
    -- 既有库迁移：backend/migrations/0012_area_lock.up.sql
    locked              INTEGER NOT NULL DEFAULT 0,
```

## 迁移文件（代码阶段产出，规格锚点）

- `backend/migrations/0012_area_lock.up.sql`：`ALTER TABLE area ADD COLUMN locked INTEGER NOT NULL DEFAULT 0;`
- `backend/migrations/0012_area_lock.down.sql`：`ALTER TABLE area DROP COLUMN IF EXISTS locked;`（SQLite ≥3.35）
- 启动自动执行，同 0007_data_dictionary 先例；幂等（重复执行不报错）。
