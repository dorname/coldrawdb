-- fix-field-tag-persistence down: 移除 field.tag 列（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE field DROP COLUMN IF EXISTS tag;
