# Delta — 技术场景总览（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 场景地图

| 编号 | 场景名称 | Phase 1 | Phase 2 | Phase 3 时序图 | API | 编排 | 状态 |
|------|---------|---------|---------|--------------|-----|------|------|
| S01 | 编辑并保存图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现** |
| S02 | 加载分享链接图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现** |
| S03 | 用户注册 / 登录 / Token 续期 | ✅ | ✅ | ✅ | ✅ | 🔲 | **后端已实现，生产前端待接入** |
| S04 | 创建 / 加入协作房间 | ✅ | ✅ | ✅ | ✅ | ✅ | **后端已实现，生产前端待接入** |
| S05 | OT 实时协作 | ✅ | ✅ | ✅ | ✅ | ✅ | **后端已实现，生产前端待接入** |
| S06 | AI 客户端通过 MCP 管理数据库图表 | ✅ | ✅ | ✅ | ✅ MCP | ✅ | **规格变更中** |

> **实现边界声明**：
> - S01/S02 前后端已实现。
> - S03～S05 的 auth/rooms/collab REST、DB、WS 与测试已实现，缺口是 `frontend-rs` 生产接入。
> - 统一 HTML 主原型模拟 S03～S05 状态，不建立真实 auth/room/WS 连接，不能作为生产前端完成证据。
> - S06 的工具设计源自 `core-S06-ai-client-mcp.md` 时序图，MVP 是独立 Rust stdio adapter。

## ADDED — S06 技术索引

| 场景 | 时序图 | Phase 2 | 工具契约 | 编排 | 实现目标 |
|---|---|---|---|---|---|
| S06 | `core-S06-ai-client-mcp.md` | `core-S06-mcp-service-design.md` | `mcp-tools.yaml` | `core-S06-mcp-service.json` | 独立 `coldrawdb-mcp` Rust 服务 |

