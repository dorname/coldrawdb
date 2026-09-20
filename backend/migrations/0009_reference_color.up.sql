-- fix-remote-github-issues-7-18（issue #12，core-01b §4.1）:
-- reference.color = 关系线颜色（'' = 默认主题色）；DEFAULT '' 自动回填存量行，向前兼容。
-- 规格原定 0006，因 0006~0008 已被占用，按序递增至 0009。

ALTER TABLE reference ADD COLUMN color VARCHAR NOT NULL DEFAULT '';
