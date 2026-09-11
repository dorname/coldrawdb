# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/test/core-S01-test-cases.md` — 登记 UT-S01-11（field.tag 保存/加载往返一致）

## [code] 代码实现
- [x] 后端 migration `0008_field_tag.up/down.sql`：`field` 表新增 `tag VARCHAR NOT NULL DEFAULT ''`（down 用 DROP COLUMN，模式照 0007）
- [x] `FieldDto` 增加 `#[serde(default)] tag` 字段 + SELECT/INSERT 接线（`backend/src/diagram_persistence.rs`）
- [x] 后端 tag 往返测试（保存→加载一致，照 0007 dict_code 测试模式）+ OpenLogos reporter 登记 UT-S01-11
