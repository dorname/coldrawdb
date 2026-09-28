# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.7 R-HIT 命中阈值改为「8 屏幕像素等价（`d * zoom <= 8.0`）」；§4.9 新增 R-HL-REF 关系线选中两端表高亮合同
- [x] D2 产出 delta `deltas/test/core-PB-relationship-test-cases.md` — UT-PB-22（命中阈值屏幕像素/zoom 感知）+ UT-PB-23（端点表高亮）+ ST-PB-13（e2e 点击关系线后两端表高亮）
- [x] D3 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 前端镜像重建注记（无 migration、无新 smoke 用例）
- [x] D4 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [ ] C1 `hit_test_reference_tier` 加 `zoom` 入参，阈值判断改为 `d * zoom <= 8.0`；所有调用点传 zoom；UT-PB-22 + reporter
- [ ] C2 `draw_canvas` 中选中关系线的端点表视为视觉高亮（`is_sel` 或独立高亮）；UT-PB-23 + reporter
- [ ] C3 e2e ST-PB-13（点击关系线 → 关系线与两端表高亮）+ 全量回归（前端 lib + spec-parity-d）

## [deploy] 部署任务

- [ ] E1 verify PASS 且用户授权后：重建 compose 前端镜像，确认 :9080 health/SPA 200

## [follow-up] 收尾

- [ ] F1 verify PASS 后带验收评论关闭 GitHub issue #46（或对应 issue 编号）
- [ ] F2 smoke PASS 后 `openlogos archive fix-issue-47-relation-hit-precision-and-highlight`，随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal 全程授权）
- [x] H2 delta 完成后 `openlogos merge fix-issue-47-relation-hit-precision-and-highlight`（/goal 授权）
- [ ] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C3 每批闭环），完成后自动提交代码
- [ ] H4 实现完成后 `openlogos verify`（/goal 授权；nice -n 10 + --test-threads=2）
- [ ] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [ ] H6 部署完成后 `openlogos smoke`（/goal 授权）
- [ ] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
