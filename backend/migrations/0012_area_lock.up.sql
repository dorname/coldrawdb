-- fix-issues-38-41-canvas-interaction（issue #41，R-AREALOCK-01）：area.locked ——
-- 区域锁定标记（防误拖动）。0 = 未锁定（存量行默认，向后兼容）；1 = 锁定
-- （画布拖动/resize no-op，选中/编辑/删除不受影响）。
-- 注：init.sql 不同步加列 —— apply_migrations 在 init_table 之后执行（同 0011 先例）。

ALTER TABLE area ADD COLUMN locked INTEGER NOT NULL DEFAULT 0;
