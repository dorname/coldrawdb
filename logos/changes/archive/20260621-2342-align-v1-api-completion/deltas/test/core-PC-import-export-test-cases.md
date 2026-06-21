## ADDED — UT-ALIGN-B01 — DiagramClient bridge config GET/PUT

- **位置**：`frontend-rs/src/editor_data_access.rs`（`get_bridge_config` / `update_bridge_config`）
- **Given**：mock HTTP 返回 envelope `{ "code": 0, "data": { "db_read_preferred": false, "db_write_enabled": true, "dual_write_local": false, "updated_at": "2026-06-21" }, "request_id": "r1" }`
- **When**：调用 `get_bridge_config()`
- **Then**：
  - 返回 `BridgeConfig` 字段与 `data` 一致
  - `db_read_preferred == false`，`db_write_enabled == true`
- **When**：调用 `update_bridge_config(&BridgeConfigUpdate { dual_write_local: Some(true), ..Default::default() })`，mock PUT 返回 200 + `{ "code": 0, "data": { "updated": true }, "request_id": "r2" }`
- **Then**：返回 `Ok(())`

## ADDED — UT-ALIGN-B02 — DiagramClient 导入日志列表与重试

- **位置**：`frontend-rs/src/editor_data_access.rs`（`list_import_logs` / `retry_import_log`）
- **Given**：mock GET `/bridge/import/local/logs` 返回 1 条 `{ id: "log-1", status: "failed", retry_count: 0, error_message: "parse error" }`
- **When**：调用 `list_import_logs(None)`
- **Then**：`Vec<ImportLogEntry>` len==1 且 `id == "log-1"`
- **When**：mock POST `/bridge/import/local/retry/log-1` 返回 `{ id: "log-1", status: "success", retry_count: 1, diagram_id: "d-new" }`
- **When**：调用 `retry_import_log("log-1")`
- **Then**：`RetryImportResponse.diagram_id == Some("d-new")` 且 `retry_count == 1`

## ADDED — UT-ALIGN-B03 — AppBar 溢出菜单与 ImportDrawer 日志区

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

## MODIFIED — 不在范围

| 条目 | 变更 |
|------|------|
| 导入任务 logs/retry UI | **移出不在范围** — 本提案 UT-ALIGN-B03 覆盖 ImportDrawer 日志区与重试按钮 |
| SQL/DBML 全屏代码视图（Phase D） | 保持不变 |
| Mermaid / PNG 导出 | 保持不变 |
