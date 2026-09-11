# 实现任务

## [delta] 规格变更
- [x] `deltas/prd/.../core-04-side-panel-tabs.md` — Areas/Notes 数据源改为 store
- [x] `deltas/test/core-SP-side-panel-test-cases.md` — UT-ALIGN-A01

## [code] 代码实现
- [x] `AreasTab` / `NotesTab` → `store.areas` / `store.notes`
- [x] 增删后 dirty + schedule_save
- [x] `impl Named for Area` / `Note`；更新 UT-SP-10
