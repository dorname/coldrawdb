# Delta — core-PB-relationship-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：范围与状态

关系工具：`Dragging` 阈值 **4px**、rubber-band、点击两点、生产确认条。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## MODIFIED / ADDED — 用例

| ID | 变更 | 合同 |
|---|---|---|
| UT-PB-06 / 06B | MODIFIED | 位移 &lt;4px → 点击（PickTarget）；≥4px → `Dragging` + `rel-rubber-band`可见 |
| UT-PB-07 | MODIFIED | 橡皮筋 path 端点=源锚点→指针；与正式关系同算法 |
| ST-PB-01 | 保留 | 点击两点 + 确认 |
| ST-PB-02 | 保留 | 拖线 + 确认 |
| ST-PB-CONFIRM（ADDED） | ADDED | 生产：落点后必须出现确认条，确认后才 `references+1`；主原型可立即 commit——生产不得要求「与原型一样立即写入」 |
| ST-PB-VIEWER（ADDED） | ADDED | Viewer 不得进入 Dragging / 建关系 |

## ADDED — 阈值纯函数（重申）

`is_relation_drag(dx,dy,threshold=4.0)` 使用欧氏距离；单位为屏幕像素（除 zoom 前）。
