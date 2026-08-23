# Delta — core-PC-import-export-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 统一原型对齐范围与状态

IO：入口经 AppBar **更多菜单** → IO 抽屉；格式 SQL/DBML/JSON（及既有 bridge 能力）。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（D 批）将 ST-PC-MENU/FMT/INSPECTOR 落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — ADDED / MODIFIED — 入口与格式

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-PC-MENU-01 | room-editor | 打开更多菜单 → 导入/导出 | 打开 IO 抽屉；非历史独立 Import 模态为主路径 | 本提案 D 批实现 |
| ST-PC-FMT-01 | 导出抽屉 | 切换 SQL/DBML/JSON | 预览随模型更新；可复制/下载（生产以规格为准） | 本提案 D 批实现 |
| ST-PC-INSPECTOR | Inspector 展开 | 打开 IO | Inspector 折叠或让位；关闭 IO 后恢复 | 本提案 D 批实现 |
| UT-PC-01～05 / ST-PC-01 | 既有 | — | 保留；入口叙述改为更多菜单→抽屉 | 既有；D 批回归 |
