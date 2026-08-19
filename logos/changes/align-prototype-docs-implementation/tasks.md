# 实现任务：对齐统一原型与生产前后端

> module: core | proposal: align-prototype-docs-implementation

## 执行约束

- 先完成本提案确认，再产出 delta；未经用户明确授权不得运行 `openlogos merge align-prototype-docs-implementation`。
- 每个代码批次必须同时包含业务代码、对应 UT/ST/e2e 测试、OpenLogos reporter 写入。
- 输出代码前必须列出本批覆盖的 UT/ST 用例 ID，并与 `logos/resources/test/*.md` 或本变更新增测试说明对齐。
- 不改变已合并的 S03/S04/S05 API、DDL 与产品设计语义；如发现契约冲突，先写明差异与修复方案。

## [delta] 规格变更
- [x] 新增或更新实现状态 delta：将 `core-implementation-checklist.md` 中 V2 生产前端待接入项拆解为 auth、rooms、collab 三批实现任务。
- [x] 新增 S03 编排测试 delta：补齐 `logos/resources/scenario/core-S03-user-auth.json` 的 register → login → me → refresh → logout → refresh 失效链路。
- [x] 更新测试矩阵 delta：补充生产前端接入用例，覆盖 `ST-PU-02`、`ST-PU-04`、`ST-PU-10`～`ST-PU-16` 对应的真实 API/WS 版本。
- [x] 更新验收说明 delta：明确统一主原型仍是视觉与交互基线，历史 S03/S04/S05 原型不作为验收入口。

## [code] 代码实现

- [x] 批次 A：S03 鉴权生产接入
  - [x] `frontend-rs/src/editor_data_access.rs` 增加 auth client：register/login/refresh/logout/me、Bearer 注入、token_expired 识别与 refresh session。
  - [x] `frontend-rs` 增加登录/注册/会话 UI 与 user-menu/session-indicator，保持 `?share=` 匿名只读链路不被拦截。
  - [x] 测试覆盖：`UT-S03-01`～`UT-S03-07` 回归、`ST-S03-01` 回归、`UT-FE-S03-01`～`UT-FE-S03-05` reporter；浏览器联调 `ST-FE-S03-01`～`ST-FE-S03-05` 标记为 e2e harness 待接入。
- [ ] 批次 B：S04 房间与邀请生产接入
  - [ ] `frontend-rs/src/editor_data_access.rs` 增加 rooms/invites/members client。
  - [ ] `frontend-rs` 增加 `/rooms`、创建房间、room-badge、邀请模态、接受邀请、成员面板、role/viewer 只读状态。
  - [ ] 测试覆盖：`UT-S04-01`～`UT-S04-10` 回归、`ST-S04-01` 回归、前端房间主链 e2e reporter。
- [ ] 批次 C：S05 WS/OT/presence 生产接入
  - [ ] `frontend-rs/src/editor_data_access.rs` 增加 WebSocket client 与 collab REST head/ops client。
  - [ ] `frontend-rs/src/editor_core.rs` 增加最小 OT op 队列、ack/serverRev、断线排队与 sync 状态。
  - [ ] `frontend-rs/src/editor_render.rs` 增加远端 cursor/presence 渲染。
  - [ ] `frontend-rs/src/editor_panels.rs` 增加 ws-status、room-presence、ot-rev、reconnect-banner、activity-feed 与 viewer 禁写。
  - [ ] 测试覆盖：`UT-C-01`～`UT-C-05` 回归、`ST-C-01` 回归、前端 WS/重连/viewer e2e reporter。
- [ ] 批次 D：全链路回归与状态收口
  - [ ] 回归 S01/S02 保存、分享、409、导入导出、命令面板、设计系统测试。
  - [ ] 更新 `logos/resources/implementation/core-implementation-checklist.md` 的完成状态。
  - [ ] 运行本地可执行测试集合并生成 OpenLogos reporter；等待用户授权后再运行 `openlogos verify`。
