# core-PC-import-export-test-cases.md

> 模块：core | 提案：redesign-phase-c-import-export, align-v1-api-completion
> 路径：`logos/resources/test/core-PC-import-export-test-cases.md`
> 最后更新：2026-06-21

## Phase C 导入/导出抽屉测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PC-01 | SQL 含 2 条 CREATE | `parse_sql_statements` | `Ok(vec![..])` len==2；摘要显示「2 条语句」 |
| UT-PC-02 | store 含 1 张表 | `export_diagram_sql(store, "generic")` | 输出含 `CREATE TABLE` 与表名 |
| UT-PC-03 | store 含表+关系 | `export_diagram_dbml(store)` | 输出含 `Table` 与 `ref:` |
| UT-PC-04 | Inspector 展开 | `open_import_drawer()` | `inspector_open==false` 且 `io_drawer==Import` |
| UT-PC-05 | DBML 含 2 个 Table 块 | `count_dbml_tables(text)` | 返回 2 |
| UT-PC-06 | 零表画布 | 点击 `guide-import-sql` | `import-drawer` 可见 |
| ST-PC-01 | 编辑器已加载 | AppBar 导入 → 粘贴 SQL → 提交 | 解析摘要可见；bridge 返回 diagramId |

### UT-ALIGN-B01 — DiagramClient bridge config GET/PUT

- **位置**：`frontend-rs/src/editor_data_access.rs`（`get_bridge_config` / `update_bridge_config`）
- **Given**：mock HTTP 返回 envelope `{ "code": 0, "data": { "db_read_preferred": false, "db_write_enabled": true, "dual_write_local": false, "updated_at": "2026-06-21" }, "request_id": "r1" }`
- **When**：调用 `get_bridge_config()`
- **Then**：
  - 返回 `BridgeConfig` 字段与 `data` 一致
  - `db_read_preferred == false`，`db_write_enabled == true`
- **When**：调用 `update_bridge_config(&BridgeConfigUpdate { dual_write_local: Some(true), ..Default::default() })`，mock PUT 返回 200 + `{ "code": 0, "data": { "updated": true }, "request_id": "r2" }`
- **Then**：返回 `Ok(())`

### UT-ALIGN-B02 — DiagramClient 导入日志列表与重试

- **位置**：`frontend-rs/src/editor_data_access.rs`（`list_import_logs` / `retry_import_log`）
- **Given**：mock GET `/bridge/import/local/logs` 返回 1 条 `{ id: "log-1", status: "failed", retry_count: 0, error_message: "parse error" }`
- **When**：调用 `list_import_logs(None)`
- **Then**：`Vec<ImportLogEntry>` len==1 且 `id == "log-1"`
- **When**：mock POST `/bridge/import/local/retry/log-1` 返回 `{ id: "log-1", status: "success", retry_count: 1, diagram_id: "d-new" }`
- **When**：调用 `retry_import_log("log-1")`
- **Then**：`RetryImportResponse.diagram_id == Some("d-new")` 且 `retry_count == 1`

### UT-ALIGN-B03 — AppBar 溢出菜单与 ImportDrawer 日志区

- **位置**：`frontend-rs/src/editor_panels.rs`（`AppBarOverflow` / `ImportDrawer` / `BridgeSettingsModal`）
- **Given**：编辑器已加载，overflow 菜单未展开
- **When**：点击 `data-testid="btn-bridge-settings"`（或等效设置项）
- **Then**：`data-testid="modal-bridge-settings"` 可见；含 `db_read_preferred` / `db_write_enabled` / `dual_write_local` 三个开关
- **When**：点击 `data-testid="btn-delete-diagram"` 且 confirm 返回 true
- **Then**：调用 `DiagramClient::delete(current_id)`；成功后跳转 `/editor`
- **When**：打开 Import 抽屉
- **Then**：`data-testid="import-logs-panel"` 可见；失败条目显示 `data-testid="import-log-retry-{id}"` 按钮
- **When**：点击 refresh（`import-logs-refresh`）
- **Then**：再次调用 `list_import_logs`

### 回归更新

| TC ID | 变更 |
|-------|------|
| UT-AB-04 | Phase C：`btn-import` **enabled**（替换 Phase A disabled 断言） |

### 不在范围

| 条目 | 变更 |
|------|------|
| SQL/DBML 全屏代码视图（Phase D） | 保持不变 |
| Mermaid / PNG 导出 | 保持不变 |

## 统一原型对齐范围与状态

IO：入口经 AppBar **更多菜单** → IO 抽屉；格式 SQL/DBML/JSON（及既有 bridge 能力）。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（D 批）将 ST-PC-MENU/FMT/INSPECTOR 落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## ADDED / MODIFIED — 入口与格式

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-PC-MENU-01 | room-editor | 打开更多菜单 → 导入/导出 | 打开 IO 抽屉；非历史独立 Import 模态为主路径 | 本提案 D 批实现 |
| ST-PC-FMT-01 | 导出抽屉 | 切换 SQL/DBML/JSON | 预览随模型更新；可复制/下载（生产以规格为准） | 本提案 D 批实现 |
| ST-PC-INSPECTOR | Inspector 展开 | 打开 IO | Inspector 折叠或让位；关闭 IO 后恢复 | 本提案 D 批实现 |
| UT-PC-01～05 / ST-PC-01 | 既有 | — | 保留；入口叙述改为更多菜单→抽屉 | 既有；D 批回归 |

## 边界

- Mermaid / PNG/PDF 等未实现格式不得标完成。
- 演示导入数据 ≠ 生产 bridge 成功。
