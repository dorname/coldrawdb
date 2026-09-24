-- diagram-api-auth down: 移除 diagram.share_token 列（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE diagram DROP COLUMN IF EXISTS share_token;
