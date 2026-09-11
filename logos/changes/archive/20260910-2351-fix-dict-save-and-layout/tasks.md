# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md` — 抽屉/检查器互斥规则 + 列表视图字典区块 + 保存链路口径修正
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 主原型对齐抽屉互斥与列表视图字典区块
- [x] 产出 delta 文件到 `deltas/test/core-S07-test-cases.md` — 新增保存往返 / 抽屉互斥 / 列表视图字典展示用例

## [code] 代码实现
- [x] 修复 `frontend-rs/src/editor_data_access.rs`：`DiagramForSave` 补 `dictionaries` 字段（保存链路）
- [x] `frontend-rs/src/editor_panels.rs`：字典抽屉与 Inspector 互斥（复用 IO 抽屉缓存/恢复模式）
- [x] `frontend-rs/src/editor_panels.rs`：ListView 新增数据字典分组展示（树区 + 详情只读）
- [x] `frontend-rs/src/styles.css`：列表视图字典区块样式
- [x] 编写对应 UT/ST 测试代码 + OpenLogos reporter（写入 `logos/resources/verify/test-results.jsonl`）
