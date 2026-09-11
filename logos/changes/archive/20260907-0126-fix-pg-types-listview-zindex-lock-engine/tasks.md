# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — MODIFIED：每引擎 MVP 类型下拉短清单权威表（PG 补 UUID/BOOLEAN、Generic 补 UUID、MySQL 不变）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — MODIFIED S04.1：增补「引擎创建后锁定」约束（AppBar 引擎下拉在房间编辑器内禁用，仅作只读标识）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型回写：`typesForDatabase` 三清单与权威表对齐 + AppBar 引擎下拉禁用
- [x] 产出 delta 文件到 `deltas/test/core-UI-modals-2-test-cases.md` — REMOVED ST-DB-01（引擎切换语义作废）+ ADDED ST-DB-02（房间编辑器引擎下拉禁用 + PG 清单含 UUID/BOOLEAN 且当前类型无占位重复）
- [x] 产出 delta 文件到 `deltas/test/core-SP-side-panel-test-cases.md` — MODIFIED ST-SP-LIST-01：补 `elementFromPoint` 绘制遮挡断言（面板中心点命中最上层元素须为面板内元素）

## [code] 代码实现
- [x] 修改 `frontend-rs/src/editor_core.rs` — `types_for_database` 常量对齐权威表（POSTGRESQL 补 UUID/BOOLEAN，GENERIC 补 UUID）
- [x] 修改 `frontend-rs/src/styles.css` — `.cdb-list-view-panel` 增加 `position: relative; z-index: 1`
- [x] 修改 `frontend-rs/src/editor_panels.rs` — AppBar `app-db-select` 在房间编辑器内禁用（只读标识）
- [x] 改写 `frontend-rs/scripts/test-spec-parity-d.mjs` — ST-DB-01 脚本替换为 ST-DB-02 断言；ST-SP-LIST-01 补 elementFromPoint 断言；OpenLogos reporter 对齐用例 ID
