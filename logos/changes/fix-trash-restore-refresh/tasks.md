# 实现任务

## [delta] 规格变更

- [ ] 产出 `deltas/test/core-S04-test-cases.md` — MODIFIED ST-S04-UI-14：断言增强（恢复后主列表必须重拉重现；mock 时序化说明）

## [code] 代码实现

- [ ] `frontend-rs/src/editor_panels.rs` — restore_room_action 成功分支触发 reload_rooms 强制刷新
- [ ] `frontend-rs/scripts/test-spec-parity-d.mjs` — installApi 动态主列表模式（恢复后动态包含），ST-S04-UI-14 初始列表不含恢复房间
- [ ] ST-S04-UI-14 单独复跑 ≥2 次 PASS 后重跑 `openlogos verify` 全量验收
