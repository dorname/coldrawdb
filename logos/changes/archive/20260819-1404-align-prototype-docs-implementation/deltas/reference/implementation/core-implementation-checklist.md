# Delta — core-implementation-checklist.md（V2 生产前端接入批次）

> module: core | proposal: align-prototype-docs-implementation

## MODIFIED — 2.1 editor_data_access

- [x] HTTP 客户端（`gloo_net::http::Request`）
- [x] diagrams API 封装（fetch_diagram / save / create / delete / import）
- [x] bridge API 封装（import / logs / retry / config）
- [x] debounce 1s 自动保存循环
- [x] 错误处理 + 指数退避重试
- [x] SaveState 状态机
- [ ] S03 auth API 客户端与 refresh/logout 接入（批次 A）
- [ ] S04 room/invite/member API 客户端接入（批次 B）
- [ ] S05 WebSocket 客户端与 collab REST head/ops 接入（批次 C）

## MODIFIED — 2.2 editor_core

- [x] Diagram 状态机（RwSignal<Diagram>）
- [x] 字段级 UndoRedoContext
- [x] dirty 标记
- [x] revision 跟踪
- [x] set_diagram / push_undo / undo / redo
- [ ] S05 最小 OT 操作队列、ack/serverRev、断线排队与 sync 状态（批次 C）

## MODIFIED — 2.3 editor_panels

- [x] Tables Tab + 列表项 + 增删改
- [x] Areas Tab
- [x] Enums Tab（V1 仅前端 state）
- [x] Notes Tab
- [x] Relationships Tab
- [x] Types Tab（V1 仅前端 state）
- [x] Issues Tab + 校验引擎
- [x] AppBar（SaveState / revision / Share / Undo / Redo）
- [x] ToolRail（6 工具按钮）
- [x] Inspector（7 Tab + 字段编辑）
- [x] ModalRoot（New / Open / Share / Rename / Settings / Confirm / Conflict）
- [x] IO Drawer（导入 / 导出，redesign-phase-c）
- [x] 全局搜索 + 类型筛选
- [x] `?share=` URL 解析 + 冷启动 GET 加载（`lib.rs` + `AppRoot`）
- [x] Command Palette 交互（Ctrl+K / 搜索 / Enter 选中，`command_palette.rs`）
- [x] Code View 交互（SQL/DBML/JSON + 复制，`code_view.rs` + AppBar 按钮）
- [x] 保存失败指数退避（3s/6s/12s，`save_with_retry`）
- [ ] S03 登录/注册/会话界面、user-menu、session-indicator 生产接入（批次 A）
- [ ] S04 房间列表、创建房间、邀请、成员、角色和 viewer 只读生产接入（批次 B）
- [ ] S05 ws-status、room-presence、ot-rev、reconnect-banner、activity-feed 生产接入（批次 C）
- [ ] Monaco wasm 完整挂载（可选升级，不阻塞本变更）

## MODIFIED — 2.4 editor_render

- [x] Canvas 容器 + 平移 + 缩放 + 框选
- [x] Table 渲染
- [x] Field 行渲染
- [x] Relationship 贝塞尔连线
- [x] Area 矩形 + 标签
- [x] Note 富文本
- [x] 选中 / 高亮 / 闪烁
- [x] 撤销栈深度指示
- [ ] S05 协作者光标、远端选中框与 presence 标签渲染（批次 C）

## MODIFIED — 7.3 编排测试

- [x] S01: 7 步骤 JSON
- [x] S02: 7 步骤 JSON
- [ ] S03: register → login → me → refresh → logout → refresh 失效 JSON（本变更新增）
- [x] S04: 房间生命周期 JSON
- [x] S05: HTTP + WebSocket OT 协作 JSON
- [x] S06: MCP stdio JSON + Rust 协议/HTTP mock 编排测试

## ADDED — 7.5 V2 生产前端接入

- [ ] 批次 A：S03 鉴权生产接入，覆盖 `UT-S03-01`～`UT-S03-07`、`ST-S03-01`、`ST-FE-S03-01`～`ST-FE-S03-05`
- [ ] 批次 B：S04 房间与邀请生产接入，覆盖 `UT-S04-01`～`UT-S04-10`、`ST-S04-01`、`ST-FE-S04-01`～`ST-FE-S04-06`
- [ ] 批次 C：S05 WS/OT/presence 生产接入，覆盖 `UT-C-01`～`UT-C-05`、`ST-C-01`、`ST-FE-S05-01`～`ST-FE-S05-06`
- [ ] 批次 D：全链路回归与状态收口，覆盖 `ST-FE-V2-01`～`ST-FE-V2-04`、S01/S02/PU 回归和 OpenLogos reporter 聚合

## MODIFIED — 9.2 V2 文档与实现状态

- [x] 设计层 3 场景设计（S03/S04/S05）；统一主原型为 `core-01-editor-prototype.html`
- [x] 技术方案层 3 场景时序（S03/S04/S05）
- [x] API 3 文件（auth.yaml + rooms.yaml + collab.yaml）
- [x] DB 3 文件（v2-auth / v2-rooms / v2-collab SQL）
- [ ] S03 编排测试 `core-S03-user-auth.json`（本变更新增）
- [x] 场景 2 编排 JSON（S04 + S05）
- [x] V2 后端实现（auth / rooms / collab REST、DB、WS 与测试）
- [ ] V2 生产前端实现（auth / rooms / WS/OT/presence，按 A/B/C 批次交付）
