-- diagram-api-auth：diagram.share_token —— S02 分享链接匿名读凭证。
-- NULL = 未开分享（匿名不可读，存量行属预期）；值存在时
-- GET /api/v1/diagrams/{id}?share_token=<值> 可匿名读（COLDRAWDB_DIAGRAMS_AUTH=on 时）。
-- 注：init.sql 不再同步加列 —— apply_migrations 在 init_table 之后执行，
-- 双份加列会在新库上撞 duplicate column；新库经本迁移直接带列。

ALTER TABLE diagram ADD COLUMN share_token TEXT;
