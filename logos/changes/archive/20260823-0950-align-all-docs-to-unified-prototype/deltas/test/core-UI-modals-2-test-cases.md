# Delta — core-UI-modals-2-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

剩余模态 / 历史边界：ImportSource、Language、SetTableWidth、ConfigureCustomTypes 等。主路径 IO 已迁抽屉后，历史 Import 模态不得再标为唯一入口。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED

| ID | 变更 | 合同 |
|---|---|---|
| UT-MM-10～14 | 保留 | 纯函数解析仍有效 |
| ST-MM-HIST-01（ADDED） | ADDED | 文档/用例须标注：历史 Import 模态为边界能力；现行主路径=更多菜单→IO 抽屉 |
| ST-MM-ESC-02（ADDED） | ADDED | 任一剩余模态 Esc/遮罩关闭后无残留层 |
| ST-MM-SCOPE（ADDED） | ADDED | remote import / 未支持语言等 V1 边界保持 Err；不得标完成 |

## ADDED — 与主原型关系

主原型未演示的次要模态：规格可保留，但第二阶段验收优先级低于 auth/rooms/editor 主链。
