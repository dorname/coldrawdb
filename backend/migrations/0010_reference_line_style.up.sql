-- fix-open-issues-19-22（#21，core-01b）：关系线条类型 / 线型
-- DEFAULT 自动回填存量行（bezier + solid），向前兼容。

ALTER TABLE reference ADD COLUMN line_type TEXT NOT NULL DEFAULT 'bezier';
ALTER TABLE reference ADD COLUMN stroke_style TEXT NOT NULL DEFAULT 'solid';
