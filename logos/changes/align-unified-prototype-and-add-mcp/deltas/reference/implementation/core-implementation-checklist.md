# Delta — core-implementation-checklist.md（状态校正 + S06 计划）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 范围

本清单追踪 S01～S06 的真实实现状态；“静态原型可演示”与“生产前端已接入”必须分别记录。

## ADDED — S03～S05 后端实现

- [x] S03 `backend/src/auth/` + `auth_v1.rs` + auth migration + 测试
- [x] S04 `backend/src/rooms/` + `rooms_v1.rs` + rooms migration + `core-S04-room-lifecycle.json`
- [x] S05 `backend/src/collab/` + `collab_v1.rs` + `/ws/rooms/{room_id}` + collab migration + `core-S05-ot-collab.json`
- [ ] S03 生产前端登录/注册/refresh/logout 接入
- [ ] S04 生产前端房间/邀请/成员/角色接入
- [ ] S05 生产前端 WS/OT/presence/重连接入

删除“V2 代码实现（auth / rooms / collab-server）未完成”和“S03 编排待补”的旧结论；S03 后端集成测试存在，但若确无 `core-S03-user-auth.json`，编排文件缺口应单独保留，不能推导为后端未实现。

## MODIFIED — 原型与文档数量

- 现行主原型：1 个 `core-01-editor-prototype.html`。
- 历史参考原型：`core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html`；不计为现行验收入口。
- 共享 CSS 与其他历史/基线 HTML 仍可保留，但统计必须注明“现行”或“历史”。

## ADDED — S06 MCP

- [ ] 独立 Rust `coldrawdb-mcp` stdio 服务
- [ ] initialize / tools/list / instructions
- [ ] 读工具：list/get/export
- [ ] 写工具：create/update/delete/import
- [ ] revision、错误映射、日志脱敏
- [ ] Claude/Codex/Cursor/OpenCode 配置
- [ ] UT-MCP-01～15、ST-MCP-01～09 + OpenLogos reporter

## MODIFIED — 关键指标口径

MCP 不计入 HTTP 端点数；单列“1 个 MCP 服务、7 个 tools、stdio transport”。diagram/bridge/auth/rooms/collab 的端点统计必须从实际 route 注册生成，不再沿用“10 + 计划值”。

