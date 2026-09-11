# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §3 关系操作：点击连线改为选中 + Inspector，删除口径收敛
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — §4.4 SQL 导入列级 DDL 解析范围 + §5.2 SQL 导出补全口径
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 连线点击不弹模态 + importModel/exportContent SQL 口径对齐
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — ST-PB-03 MODIFIED + ST-PB-04 补 Backspace 断言
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — UT-PC-02 MODIFIED + 新增 UT-PC-07/UT-PC-08

## [code] 代码实现
- [x] `frontend-rs/src/editor_panels.rs`：on_reference_pick 移除弹模态；删除关系详情模态组件；`parse_sql_import_tables` 重写为列级 DDL 解析（含 REFERENCES → references）
- [x] `frontend-rs/src/editor_panels.rs::export_diagram_sql` 补全 UNIQUE/自增/DEFAULT/COMMENT/FK
- [x] 编写/更新 UT（UT-PC-02/07/08）与 e2e（ST-PB-03/04）对齐代码，含 OpenLogos reporter 写入
