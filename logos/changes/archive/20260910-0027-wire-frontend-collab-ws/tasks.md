# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/test/core-V2-production-frontend-test-cases.md` — §4 新增「真实 WS 传输接线」子节：UT-FE-S05-13~17（op 映射 / remote_op 应用 / checkpoint 头 / viewer 拦截 / 重连策略）+ ST-FE-S05-07~09（Playwright 双上下文真实后端：A→B 500ms 可见、断网队列恢复 flush、presence 远端光标）
- [x] 产出 delta 文件到 `deltas/test/core-S05-test-cases.md` — 更新 §1 状态行（生产前端已接入真实 WS）
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md` — 更新 §0 生产状态行（前端接入完成）

## [code] 代码实现
- [x] 前端新增 CollabClient WS 模块（web_sys WebSocket：connect/send/onmessage/onclose，`build_ws_url` 接线）
- [x] `editor_panels.rs` 房间进入 effect 接线真实 WS（替换"仅 get_head 后置 Connected"的假连接）
- [x] 本地编辑 → CollabOp 映射（table/field/reference/area/note 15 种 type），optimistic apply + 上行 + ack 对账
- [x] remote_op → EditorStore 应用 + 画布渲染 + activity-feed
- [x] presence 上行（节流）+ 远端光标渲染（排除本人）
- [x] 断线重连：Banner、可见离线队列、指数退避 ≤5 次、sync 补发/全量 snapshot、token_expired refresh 重连
- [x] checkpoint：room 内 PUT diagram 携带 `X-Room-Id` / `X-Collab-Server-Rev`；`COLLAB_REV_STALE` 静默重同步，不弹 S01 409 模态
- [x] viewer 只读：拦截 op 上行，保留接收
- [x] 核对后端 diagrams PUT 的 collab 头处理，缺失则补齐（含 `COLLAB_REV_STALE`）
- [x] 编写对应 UT/ST 测试代码并登记 OpenLogos reporter
