# 实现任务

## [delta] 规格变更

- [ ] 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`：
  - 在 §5.10 新增 R-WIDTH-06（effective 字号下 auto 宽度重测）、R-WIDTH-07（行高/表头高随 effective 字号自适应）、R-WIDTH-08（命中/锚点/精灵缓存与绘制同口径）。
  - 在 §5.12 新增 R-LOD-09（表维度下表头高随表名字号补偿自适应）。
  - 在 §5.16 新增 R-FONT-07（字号倍率/钳制变化时触发重测与重光栅）。
- [ ] 产出 delta `deltas/test/core-CR-canvas-test-cases.md`：新增 UT-CR-FONT-02、ST-CR-FONT-02。
- [ ] 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`：前端镜像重建注记（无 migration、无新 smoke 用例）。
- [ ] 读回验证：从磁盘读回全部 delta 文件确认落盘。

## [code] 代码实现

- [ ] C1 新增 `scaled_field_row_height(zoom, label_scale)` / `scaled_table_header_height(...)` 纯函数并补 UT。
- [ ] C2 扩展 `estimate_content_width_for(table, comment_mode, zoom, label_scale, tier)`，按 effective 字号重算 auto 宽；保持 `estimate_content_width` 作为 `zoom=1.0, label_scale=1.0, tier=Detail` 的兼容包装。
- [ ] C3 扩展 `compute_table_render_size_for(table, comment_mode, zoom, label_scale, tier)` 以动态行高/表头高；保持 `compute_table_render_size` 作为兼容包装。
- [ ] C4 让 `resolve_table_width_for` / `field_anchor_for_side` / `anchor_for_tier` 使用新的 effective 字号宽度；命中测试、关系锚点、精灵缓存尺寸同源。
- [ ] C5 更新 `table_sprite_fingerprint` 混入 effective font clamp bucket（避免同一 zoom bucket 内不同 zoom 字号不一致）。
- [ ] C6 更新 `draw_table_body` / `draw_table_topology_body` 使用动态表头高/行高，保持文本垂直居中。
- [ ] C7 新增/更新 UT-CR-FONT-02 + reporter 登记。
- [ ] C8 新增 ST-CR-FONT-02 到 `scripts/test-spec-parity-d.mjs` + reporter 登记 + 全量回归（前端 lib + spec-parity-d）。

## [deploy] 部署任务

- [ ] E1 verify PASS 且用户授权后：重建 compose 前端镜像，确认 :9080 health/SPA 200。

## [follow-up] 收尾

- [ ] F1 verify PASS 后带验收评论关闭对应 GitHub issue。
- [ ] F2 smoke PASS 后 `openlogos archive fix-font-lod-consistency-and-table-resize`，随后 git push。

## 人类确认点

- [ ] H1 用户确认本提案后再产出 delta（/goal 全程授权）
- [ ] H2 delta 完成后 `openlogos merge fix-font-lod-consistency-and-table-resize`（/goal 授权）
- [ ] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C8 每批闭环），完成后自动提交代码
- [ ] H4 实现完成后 `openlogos verify`（/goal 授权；nice -n 10 + --test-threads=2）
- [ ] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [ ] H6 部署完成后 `openlogos smoke`（/goal 授权）
- [ ] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
