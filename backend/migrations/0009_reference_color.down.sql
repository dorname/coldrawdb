-- fix-remote-github-issues-7-18 down: 移除 reference.color 列（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE reference DROP COLUMN IF EXISTS color;
