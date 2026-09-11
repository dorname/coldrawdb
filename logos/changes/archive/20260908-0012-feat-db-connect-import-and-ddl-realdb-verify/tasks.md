# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md` — 连接数据库导入能力（端点行为、introspection 口径、错误语义、安全边界）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — ImportDrawer「数据库连接」来源交互 + DDL 真实库验证口径
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — IA 同步（导入来源新增「数据库连接」）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型：导入抽屉「数据库连接」来源（引擎下拉 + 连接参数 + 解析预览复用）
- [x] 产出 delta 文件到 `deltas/api/bridge.yaml` — 新增 `POST /api/v1/bridge/import/connect`
- [x] **验证 API YAML** — `logos/resources/api/bridge.yaml` 有效 YAML + OpenAPI 3.0.3，5 paths，新端点与双 schema 均在（python yaml 校验通过）
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — 新增用例 UT-PC-11～15 / ST-PC-04（全部落在用例总表表行）
- [x] ~~产出 delta 文件到 `deltas/test/core-PV-verify-pre-run-test-cases.md`~~ — **无需变更**：6 个新 ID 均为表行（ledger 校验器可直接扫描），非标题式用例

## [code] 代码实现
- [x] `backend/src/phase3_bridge.rs` + 新模块：`POST /api/v1/bridge/import/connect`（sqlx SQLite/PG introspection → 方言 DDL），含后端 UT
- [x] `frontend-rs/src/editor_panels.rs`：ImportDrawer「数据库连接」来源 UI + 接线（复用本地合并管道），含 UT 锚点
- [x] `frontend-rs/tests/`：DDL 真实库验证集成测试（嵌入式 PG + SQLite 执行导出 DDL），dev-dependencies 引入 `sqlx`(sqlite) + `postgresql_embedded`
- [x] e2e `scripts/test-spec-parity-d.mjs`：连接数据库导入 ST 用例（mock 端点），含 OpenLogos reporter 写入
