# Delta — core-S04-test-cases.md（fix-trash-restore-refresh）

> 模块：core | 场景：S04 | 关联 issue #75：回收站恢复成功后主列表不刷新

## MODIFIED — ST-S04-UI-14 — 回收站恢复（断言增强）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（既有用例增强，编号不变）。
- **MODIFIED 断言**：恢复成功后，主房间列表必须**重新拉取**并包含恢复的房间——
  installApi 启用动态主列表模式：初始 `roomsList` 不含 room-arch（真实恢复前
  状态），恢复请求处理后 `GET /api/v1/rooms` 动态包含 room-arch。断言
  「返回列表后房间重现」在静态 mock 下不再恒真，真正依赖 reload 刷新。
- **不变断言**：恢复调 `POST /rooms/{id}/restore`；Toast「已恢复房间」；条目移出回收站。
- **reporter**：真实运行后登记 `ST-S04-UI-14`（编号沿用，不新增 ID）。
