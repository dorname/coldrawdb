-- fix-field-tag-persistence（字段 tag 持久化缺口修复）:
-- field.tag = 字段标签（ListView「说明」列 / ByTag 分组键，core-01a），
-- 空串 = 无标签；DEFAULT '' 自动回填存量行，向前兼容。

ALTER TABLE field ADD COLUMN tag VARCHAR NOT NULL DEFAULT '';
