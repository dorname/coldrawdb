# Delta — core 前后端对齐验收说明

> module: core | proposal: align-prototype-docs-implementation

## ADDED — 1. 验收边界

`core-01-editor-prototype.html` 是 S01～S05 的唯一现行视觉与交互基线，但它是静态单文件原型，不调用生产 API。本变更的生产验收必须运行 `frontend-rs` 与 `backend`，并验证真实 REST/WS 链路。

历史原型 `core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html` 仅用于差异追溯，不作为本变更验收入口。

## ADDED — 2. 生产验收标准

- **FEALIGN-AC-01 鉴权闭环**：登录/注册/refresh/logout/me 全部通过真实 `/api/v1/auth/*`；access token 不写 localStorage；refresh 失败不无限重试。
- **FEALIGN-AC-02 分享兼容**：未登录用户打开 `?share=` 仍可走 S02 匿名只读加载，不被 S03 auth guard 阻断。
- **FEALIGN-AC-03 房间闭环**：`/rooms`、创建 room、生成 invite、preview、accept、成员 role 更新和移除全部调用真实 `/api/v1/rooms*`。
- **FEALIGN-AC-04 权限一致**：viewer 的 ToolRail、Inspector、邀请入口和 WS op 均保持只读；后端 READ_ONLY/FORBIDDEN 错误能映射到用户可见提示。
- **FEALIGN-AC-05 实时协作**：room 编辑器能建立 `/ws/rooms/{roomId}?token=...`，处理 connected/ack/remote_op/presence/sync/error 帧，并显示 ws-status、ot-rev、room-presence、reconnect-banner。
- **FEALIGN-AC-06 断线不丢编辑**：断线期间本地 op 进入队列，恢复后 sync 并清零；重连失败时明确降级为本地编辑并提示 409 风险。
- **FEALIGN-AC-07 V1 不回退**：S01 保存、409 冲突、S02 分享加载、IO 抽屉、命令面板、设计系统和移动端布局继续通过既有测试。
- **FEALIGN-AC-08 Reporter 完整**：每个新增 UT/ST/e2e 用例写入 OpenLogos reporter；失败信息脱敏。

## ADDED — 3. 源码实现限制

- 不新增与已合并 API/DDL 冲突的字段、端点或表。
- 不把 refresh token、cookie 或 access token 原文写入日志、reporter 或截图文件名。
- 不直连 SQLite 绕过后端 API 实现前端功能。
- 不把静态原型中的模拟状态直接标记为生产实现。
- 不在未完成批次中提前勾选 `core-implementation-checklist.md`。

## ADDED — 4. verify 前检查

运行 `openlogos verify align-prototype-docs-implementation` 前，至少应完成：

- 后端 auth/rooms/collab Rust 测试回归。
- 前端 Rust 单元测试回归。
- 前端 Playwright/e2e 覆盖 S03/S04/S05 生产接入主链。
- 统一原型 ST-PU-01～ST-PU-19 回归，确认视觉交互基线未破坏。
- reporter 中新增用例 ID 与 `core-V2-production-frontend-test-cases.md` 一一对应。
