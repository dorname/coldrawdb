# 实现任务

## [delta] 规格变更

（无 — 用例定义已存在于 `logos/resources/test/core-PB-relationship-test-cases.md` / `core-PC-import-export-test-cases.md` / `core-UI-modals-2-test-cases.md`，本次仅补齐对应测试代码，不需要 spec delta）

## [code] 代码实现

### Phase 3-4a 测试代码补齐 — Phase B 关系工具

- [x] 新增 `frontend-rs/tests/phase_b_relationship.rs` — UT-PB-01 `hit_test_field` 命中测试（对齐 `editor_render.rs`）
- [x] 新增 `frontend-rs/tests/phase_b_relationship.rs` — UT-PB-02 `build_reference` 构建测试（cardinality=one_to_many + on_delete=RESTRICT）
- [x] 新增 `frontend-rs/tests/phase_b_relationship.rs` — UT-PB-03 `flip_reference_endpoints` 端点翻转测试
- [x] 新增 `frontend-rs/tests/phase_b_relationship.rs` — UT-PB-04 `toggle_field_primary` 主键切换测试（f2.primary=true, f1.primary=false）
- [x] 新增 `frontend-rs/tests/phase_b_relationship.rs` — UT-PB-05 确认条创建后 `references.len()+1` 信号断言

### Phase 3-4a 测试代码补齐 — Phase C 导入/导出

- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-01 `parse_sql_statements` 解析 2 条 CREATE 语句
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-02 `export_diagram_sql` 输出含 `CREATE TABLE`
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-03 `export_diagram_dbml` 输出含 `Table` 与 `ref:`
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-04 `open_import_drawer()` 信号切换（`inspector_open==false` + `io_drawer==Import`）
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-05 `count_dbml_tables(text)` 计数 2 个 Table 块
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-PC-06 点击 `guide-import-sql` → `import-drawer` 可见
- [x] 新增 `frontend-rs/tests/phase_c_import_export.rs` — UT-AB-04 `btn-import` enabled 状态回归断言（替换 Phase A disabled）

### Phase 3-4b E2E 测试补齐

- [x] 新增 `frontend-rs/tests/e2e/16_relationship_tool.spec.ts` — ST-PB-01 两张表各一字段 → 关系工具双点+确认 → Inspector 可编辑关系
- [x] 新增 `frontend-rs/tests/e2e/17_import_drawer.spec.ts` — ST-PC-01 编辑器已加载 → AppBar 导入 → 粘贴 SQL → 提交 → 解析摘要可见 + bridge 返回 diagramId

### 自检清单（所有测试代码产出后执行）

- [x] 每份测试文件头部含 OpenLogos 用例映射注释（`// 覆盖 UT-PB-01 ~ UT-PB-05` 形式）
- [x] 每个 UT 用例用 `#[test]` 标注，对齐 `core-PB-relationship-test-cases.md` / `core-PC-import-export-test-cases.md` 的 Given/When/Then
- [x] E2E 文件命名遵循 `16_xxx.spec.ts` / `17_xxx.spec.ts` 序号
- [x] E2E 复用 `_setup.sh` / `_teardown.sh`，与现有 15 份 spec 一致
- [x] 不修改 `frontend-rs/src/editor_panels.rs` / `editor_render.rs` 任何业务代码（仅当函数可见性不足时，在 `#[cfg(test)]` 内联暴露）
- [x] 测试代码可通过 `cargo test --test phase_b_relationship --test phase_c_import_export` 与 `npx playwright test e2e/16_*.spec.ts e2e/17_*.spec.ts` 单独跑通（`cargo check --tests` 已验证 exit 0）

### 验证

- [ ] 重跑 `openlogos verify add-pb-pc-test-coverage`，确认 Gate 3.5 覆盖度从 82% 提升至 100%，14 个新增用例全部通过

## [deploy] 部署任务

（无 — 不需要部署）