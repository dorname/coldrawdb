-- fix-issues-38-41-canvas-interaction down: 移除 area.locked 列（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE area DROP COLUMN IF EXISTS locked;
