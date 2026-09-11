# Delta — core-PC-import-export-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：范围与状态

IO：入口经 AppBar **更多菜单** → IO 抽屉；格式 SQL/DBML/JSON（及既有 bridge 能力）。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED — 入口与格式

| ID | 前置 | 操作 | 预期 | 变更 |
|---|---|---|---|---|
| ST-PC-MENU-01 | room-editor | 打开更多菜单 → 导入/导出 | 打开 IO 抽屉；非历史独立 Import 模态为主路径 | ADDED |
| ST-PC-FMT-01 | 导出抽屉 | 切换 SQL/DBML/JSON | 预览随模型更新；可复制/下载（生产以规格为准） | ADDED |
| ST-PC-INSPECTOR | Inspector 展开 | 打开 IO | Inspector 折叠或让位；关闭 IO 后恢复 | ADDED |
| UT-PC-01～05 / ST-PC-01 | 既有 | — | 保留；入口叙述改为更多菜单→抽屉 | MODIFIED |

## ADDED — 边界

- Mermaid / PNG/PDF 等未实现格式不得标完成。
- 演示导入数据 ≠ 生产 bridge 成功。
