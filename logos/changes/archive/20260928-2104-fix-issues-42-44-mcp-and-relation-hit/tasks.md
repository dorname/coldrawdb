# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 新增关系线命中合同（线型同源 + 最近距离优先 + 阈值外不命中，#44）与悬浮 tooltip 交互规格（源表.源字段 → 目标表.目标字段，#43）
- [x] D2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — update_diagram 错误可诊断合同（status/Content-Type/body 摘要）+ USE_OP_CHANNEL 非 JSON 鲁棒识别 + import_schema「仅新建」文案（#42）
- [x] D3 产出 delta `deltas/test/core-PB-relationship-test-cases.md` — #44：UT（最近距离优先/线型同源/阈值外不命中）+ ST（密集线点击目的一致）；#43：UT（tooltip 文案纯函数）+ ST（悬浮出 tooltip、不改选中态）
- [x] D4 产出 delta `deltas/test/core-S06-test-cases.md` — #42：UT（非 JSON 409 含 USE_OP_CHANNEL 字样仍走 op 通道 / 错误 details 含 status+content_type+body_excerpt / import_schema 描述文案）
- [x] D5 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 前端镜像随 compose 重建部署注记（无 migration、无新 smoke 用例）
- [x] D6 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [x] C1 #44：`hit_test_reference_tier` 线型同源 + 最近距离优先（bezier/orthogonal 双几何，阈值内取 min 距离）；UT + ST + reporter
- [x] C2 #43：hover 检测（pointermove 未拖拽）+ tooltip 渲染（复用 C1 命中函数）；UT + ST + reporter
- [x] C3 #42：mcp-server `request()` 错误 details 携带 status/content_type/body_excerpt；409 非 JSON 含 USE_OP_CHANNEL 字样走 op 通道；import_schema tool 描述「仅新建」；UT + reporter
- [x] C4 e2e 全量回归（spec-parity-d）+ 前端 lib 全量 + mcp-server 测试全绿

## [deploy] 部署任务

- [x] E1 verify PASS 且用户授权后：按部署方案重建 compose 前端镜像（mcp-server 不经 compose），确认服务可用（:9080 health/SPA 200）—— 已完成：coldrawdb:v1 重建+容器 healthy，health 200 / SPA 200

## [follow-up] 收尾

- [x] F1 verify PASS 后带验收评论关闭 GitHub issue #42 / #43 / #44 —— 已评论+关闭
- [x] F2 smoke PASS 后 `openlogos archive fix-issues-42-44-mcp-and-relation-hit`，随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权 + 用户「授权给你」）
- [x] H2 delta 完成后 `openlogos merge fix-issues-42-44-mcp-and-relation-hit`（/goal 授权）
- [x] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C4 每批闭环），完成后自动提交代码
- [x] H4 实现完成后 `openlogos verify`（Gate 3.6 PASS：512 通过/0 失败/34 跳过，覆盖 546/546）（/goal 授权；nice -n 10 + --test-threads=2）
- [x] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [x] H6 部署完成后 `openlogos smoke`（/goal 授权）—— Gate 3.8 PASS 10/10（首轮 9 unknown 系残留 dev backend 占 3000，清理后重跑全绿）
- [x] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
