# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 在 R-WIDTH-08 / R-LOD-08 中明确「同口径」包含当前 `comment_mode`
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT-CR-ANCHOR-01、ST-CR-ANCHOR-01
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 部署注记

## [code] 代码实现
- [ ] 修改 `frontend-rs/src/editor_render.rs`：
  - `field_anchor_for_side_with` / `anchor_for_tier_with` 接收 `comment_mode` 并用于 `resolve_table_width_for`
  - `pick_port_sides` 增加 `comment_mode` 参数并透传
  - `calc_path_tier_with` / `calc_orthogonal_path_tier_with` / `calc_straight_path_tier_with` 透传 `comment_mode`
  - 更新所有调用点（draw_canvas、hit_test 系列、path 计算）传入当前 `comment_mode`
- [ ] 新增 `UT-CR-ANCHOR-01` 单元测试
- [ ] 新增 `ST-CR-ANCHOR-01` e2e 测试到 `frontend-rs/scripts/test-spec-parity-d.mjs`
- [ ] 更新 `frontend-rs/tests/openlogos_reporter.rs` 登记 UT-CR-ANCHOR-01

## [deploy] 部署任务
- [ ] verify PASS 且用户授权后：重建 compose 前端镜像，确认 :9080 health/SPA 200
