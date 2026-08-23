# Delta — core-SP-side-panel-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

Inspector 锚点与响应式抽屉。验收锚点：**`data-testid="inspector"`**（禁止仅用 `inspector-panel`）。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED

| ID | 变更 | 合同 |
|---|---|---|
| UT-SP-ANCHOR-01（ADDED） | ADDED | 生产 DOM 存在 `inspector`；选中表后面板字段可编辑（可写角色） |
| ST-SP-RESP-01（ADDED） | ADDED | ≤720px：Inspector 以抽屉/叠层呈现；`btn-inspector-toggle` 可开关；关闭后不挡画布 |
| ST-SP-RESP-02（ADDED） | ADDED | 桌面三列与窄屏单列切换后，不残留错误 layout class |
| UT-SP-02/09/10 | 保留 | Tab 搜索/切换；与统一壳层共存 |
| ST-SP-VIEWER（ADDED） | ADDED | Viewer：Inspector 只读 |

## ADDED — 与 IO / 成员抽屉互斥

同时打开多个侧层时，必须有可关闭路径；不得出现不可恢复遮挡（对齐 ST-PU-17）。
