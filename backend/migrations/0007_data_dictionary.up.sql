-- feat-data-dictionary（S07 数据字典）:
-- diagram.dictionaries = 字典数组 JSON blob（NULL = 无字典，向前兼容）。
-- field.dict_code = 字段绑定的字典编码（软引用，空串 = 未绑定）。

ALTER TABLE diagram ADD COLUMN dictionaries TEXT;
ALTER TABLE field ADD COLUMN dict_code VARCHAR NOT NULL DEFAULT '';
