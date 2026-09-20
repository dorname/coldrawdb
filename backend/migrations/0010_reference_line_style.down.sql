-- fix-open-issues-19-22 down：移除 reference.line_type / stroke_style

ALTER TABLE reference DROP COLUMN IF EXISTS line_type;
ALTER TABLE reference DROP COLUMN IF EXISTS stroke_style;
