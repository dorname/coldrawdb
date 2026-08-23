# Delta — core-PU-unified-prototype-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围与说明

> module: core | proposal: align-all-docs-to-unified-prototype | type: prototype acceptance

唯一现行主原型：`core-01-editor-prototype.html`。本矩阵验证 **auth → rooms → invite → room-editor** 完整交互与视觉基线；历史 `core-03/04/05-*-prototype.html` 不纳入现行验收。

静态原型不调用生产 API。生产语义对齐见 `core-V2-production-frontend-test-cases.md`；状态表述：后端已实现；生产前端部分接入；逐项对齐待第二阶段（代码实现由 `implement-unified-prototype-spec-parity` 承接）。

实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED — 关键 `data-testid` 锚点合同（与主原型一致）

| 页面 / 区域 | 关键 `data-testid` |
|---|---|
| auth | `login-form` / `register-form` |
| rooms | `rooms-list-page` / `room-list` / `btn-create-room` |
| invite | `invite-accept-page` / `btn-accept-invite` |
| room-editor 壳 | `room-editor-page` / `app-bar` / `tool-rail` / `editor-canvas` / `inspector` / `status-bar` |
| 保存 / 协作 | `save-state` / `revision-display` / `ws-status` / `ot-rev` / `room-presence` / `reconnect-banner` |
| 浮层 / IO | `tool-search` / `code-view-modal` / `btn-more-menu` / `btn-import` / `btn-export` / `room-members-panel` |
| 关系 / 主题 | `rel-rubber-band` / `btn-theme-toggle` |

## ADDED — 统一原型对齐用例增量与修订索引

| ID | 变更 | 说明 |
|---|---|---|
| ST-PU-01～04 | MODIFIED | 明确页面流 auth→rooms→room-editor；断言关键 testid 存在 |
| ST-PU-05 | MODIFIED | 原型松手网格为演示 `GRID=12`；**生产合同**为 `GRID_SIZE=20`（见 CR）；拖动中关系 `path[d]` 跟手 |
| ST-PU-06 / 20 | 保留 | 点击两点与拖线（`DRAG_THRESHOLD=4`）双路径 |
| ST-PU-09 | MODIFIED | 主题切换后无残留 overlay；暗色 token 生效 |
| ST-PU-17 | MODIFIED | 桌面 + ≤720px：Inspector/成员/IO/模态关键操作可达，无横向溢出 |
| ST-PU-18 | 保留 | `prefers-reduced-motion` |
| ST-PU-22（ADDED） | ADDED | 未登录打开主原型默认入口 → 仅 auth；不出现私有 `room-list` 数据 |
| ST-PU-23（ADDED） | ADDED | 邀请失效态：`invite-accept-page` 无加入按钮 |
| ST-PU-24（ADDED） | ADDED | room-editor 可见 `ws-status`、`ot-rev`、`room-presence`（演示值须标「演示」） |

## ADDED — 视觉 / 主题 / 响应式基线

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-PU-22 | 冷启动 | 打开主 HTML | 落在 auth；关键 login/register testid 存在 |
| ST-PU-23 | 邀请失效演示 | 打开 invite 失效路径 | 说明文案可见；无 accept 主按钮 |
| ST-PU-24 | 已进入 room-editor | 观察 StatusBar / AppBar | `ws-status`、`ot-rev`、`room-presence` 可见；演示控件标注演示 |
| ST-PU-25 | 编辑器 | 切换主题 | `data-mode` 切换；画布/壳层对比度可读；无半透明残留层 |
| ST-PU-26 | 720px | 开关 Inspector 与 IO | 以抽屉呈现；可关闭；不锁定背景滚动（或关闭后恢复） |

## ADDED — PU-AC 追溯补充

| 验收标准 | 覆盖用例 |
|---|---|
| PU-AC-09 页面流四态 | ST-PU-22～24、02、04、10 |
| PU-AC-10 主题与响应式 | ST-PU-09、17、18、25、26 |

## ADDED — 统一原型对齐补充：边界声明

- 主原型可演示 ≠ 生产前端逐项完成。
- 生产对齐状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。
