# 实现任务

## [delta] 规格变更

- [x] 变更范围已在 proposal.md 记录（无独立 delta 文档，行为对齐既有 OpenAPI）

## [code] 代码实现

### Batch 1 — diagram 实体持久化
- [x] `backend/src/diagram_persistence.rs` load/save/persist_import_payload
- [x] `diagrams_v1.rs` GET/PUT/import 接线
- [x] `editor_data_access.rs` DiagramOut / DiagramForSave 完整嵌套

### Batch 2 — bridge 导入解析
- [x] `phase3_bridge.rs` import 后 persist_import_payload
- [x] `parse_sql_import_tables` + `build_import_payload` SQL 路径
- [x] `parse_dbml_import_tables` + DBML payload 含 tables

### Batch 3 — 冲突/撤销/New 接线
- [x] ConflictDialog force/reload handlers
- [x] CommandStack revert/execute + UndoRedoButtons + KeyboardShortcuts
- [x] ModalRoot New/Rename 接 create/rename

### Batch 4 — 原型 UI parity
- [x] AppBar Logo → Undo/Redo → 标题 → save-state → IO pill
- [x] Inspector 列挂载 LeftPanel（7 Tab）+ RightPanel（field-editor）
- [x] IoDrawer 外层 `io-drawer` testid
- [x] ShareModal `share-url`；搜索框 `side-search`

## [verify]

- [ ] `bash scripts/run-verify-tests.sh`
- [ ] 用户授权后 `openlogos verify fix-diagram-entity-persistence`
