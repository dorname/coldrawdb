# Delta — core-S04-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

S04：房间列表、创建、邀请、成员、Viewer。页面锚点对齐主原型 `rooms-list-page` / `invite-accept-page` / `room-editor-page`。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（B 批）将 UI / 页面流用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — UI / 页面流用例

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S04-UI-01 | 已登录 | 打开 `/rooms` | `room-list` 或空状态；`btn-create-room`；用户菜单 | 本提案 B 批实现 |
| ST-S04-UI-02 | 已登录 | 创建房间 | `POST /rooms`；进入 room-editor；`room-badge` 显示房间名 | 本提案 B 批实现 |
| ST-S04-UI-03 | Owner | 生成邀请 | 显示 invite URL；preview/accept 链路可用 | 本提案 B 批实现 |
| ST-S04-UI-04 | 另一用户 | 接受邀请 | 加入后进入同一 room-editor | 本提案 B 批实现 |
| ST-S04-UI-05 | Owner | 成员面板改角色/移除 | 列表即时更新；API PATCH/DELETE | 本提案 B 批实现 |
| ST-S04-UI-06 | Viewer | 新建表/改字段/邀请 | 写操作禁用或拦截；无写 API/WS op；只读提示 | 本提案 B 批实现 |
| ST-S04-UI-07 | 邀请过期 | 打开 invite | 失效页；无加入按钮 | 本提案 B 批实现 |

## MODIFIED — 既有 S04 用例补充约束

后端编排保持；前端验收必须使用上表 UI 用例，不得仅以 API 200 视为「已对齐主原型」。本提案 B 批负责落实上表自动化。
