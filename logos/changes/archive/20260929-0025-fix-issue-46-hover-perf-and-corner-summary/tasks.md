# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.8 R-HOV 修订为 B1 角落摘要区 + 新增 R-PERF-HOV（守卫/节流/信号瘦身）
- [x] D2 产出 delta `deltas/test/core-PB-relationship-test-cases.md` — UT（hover 守卫/角落定位锚点）+ ST-PB-12 口径修订（角落摘要，不再断言光标跟随坐标）
- [x] D3 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 前端镜像重建注记（无 migration、无新 smoke 用例）
- [x] D4 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [x] C1 hover 信号瘦身+守卫：hover_ref 只存 ref_id，同 ref 命中不反复 set；tooltip 改角落固定定位（CSS absolute 锚定 canvas-stack，去掉 client 坐标绑定）；UT + reporter
- [x] C2 pointermove hover 命中 rAF 节流（每帧至多一次 hit_test）；UT + reporter
- [x] C3 e2e 修订 ST-PB-12（角落摘要口径）+ 全量回归（前端 lib + spec-parity-d）

## [deploy] 部署任务

- [x] E1 verify PASS 且用户授权后：重建 compose 前端镜像，确认 :9080 health/SPA 200 —— 已完成：health 200 / SPA 200 / 容器 healthy

## [follow-up] 收尾

- [x] F1 verify PASS 后带验收评论关闭 GitHub issue #46 —— 已评论+关闭
- [x] F2 smoke PASS 后 `openlogos archive fix-issue-46-hover-perf-and-corner-summary`，随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权）
- [x] H2 delta 完成后 `openlogos merge fix-issue-46-hover-perf-and-corner-summary`（/goal 授权）
- [x] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C3 每批闭环），完成后自动提交代码
- [x] H4 实现完成后 `openlogos verify`（Gate 3.6 PASS：516 通过/0 失败/34 跳过，覆盖 550/550）（/goal 授权；nice -n 10 + --test-threads=2）
- [x] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [x] H6 部署完成后 `openlogos smoke`（/goal 授权）—— Gate 3.8 PASS 10/10
- [x] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
