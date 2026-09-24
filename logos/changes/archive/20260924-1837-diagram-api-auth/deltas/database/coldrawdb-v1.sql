# Delta — coldrawdb-v1.sql（diagram-api-auth）

> 模块：core | 提案：diagram-api-auth
> 目标：`diagram` 表新增 `share_token`（可空）——S02 分享链接匿名读的凭证列；存量行 NULL = 未开分享，匿名不可读（预期行为）。

## MODIFIED — 2. diagram 表定义

```sql
CREATE TABLE IF NOT EXISTS diagram (
    id              TEXT    PRIMARY KEY,
    title           TEXT    NOT NULL,
    revision        INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- diagram-api-auth：分享令牌（S02 匿名读凭证）。NULL = 未开分享；
    -- 值存在时 GET /api/v1/diagrams/{id}?share_token=<值> 可匿名读（flag=on 时）。
    share_token     TEXT
);
CREATE INDEX IF NOT EXISTS idx_diagram_updated_at ON diagram(updated_at DESC);
```

## ADDED — 迁移注记

```sql
-- @migration backend/migrations/0011_diagram_share_token.up.sql：
-- ALTER TABLE diagram ADD COLUMN share_token TEXT（存量行 NULL=未开分享，匿名不可读，属预期）；
-- down 迁移 DROP COLUMN share_token。幂等（重复执行不报错）。
-- 另：backend/init.sql 的 diagram DDL 同步加列（新库直接带列）。
```
