-- feat-data-dictionary down: 移除字典两列（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE diagram DROP COLUMN IF EXISTS dictionaries;
ALTER TABLE field DROP COLUMN IF EXISTS dict_code;
