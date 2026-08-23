# Delta — core-PE-design-system-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：范围与状态

Design System：token / icon / 组件 / dark / motion，须与主原型壳层（auth/rooms/room-editor）一致。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED — 对齐合同

| ID | 变更 | 合同 |
|---|---|---|
| UT-E1-01/02 | MODIFIED | token 在 auth/rooms/editor 三页可读；`:root` 与 `[data-mode=dark]` 完整 |
| UT-E2-* | 保留 | 图标注册；关键工具/状态图标在主链可见 |
| UT-E3-* | MODIFIED | Button/Modal/Drawer/Tag 等用于统一壳层；关闭行为无残留 |
| UT-E5-* / ST-PE-06 | MODIFIED | 主题切换覆盖 auth→rooms→editor；刷新保持策略按规格 |
| UT-E6-* / ST-PE-07 | MODIFIED | Toast/抽屉/光标 motion；`prefers-reduced-motion` 降级 |
| ST-PE-SHELL-01（ADDED） | ADDED | 主原型视觉基线对照：间距/圆角/玻璃层级不回退到历史独立原型风格 |
| ST-PE-CONTRAST-01（ADDED） | ADDED | 暗色下 StatusBar `ws-status`/`ot-rev` 与 AppBar 保存态可读 |

## ADDED — 第二阶段说明

像素级视觉回归与全量 HP 截图：规格合同已立；执行与对比基线更新标为**待第二阶段**。
