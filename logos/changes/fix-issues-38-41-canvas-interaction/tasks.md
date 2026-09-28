# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 新增 §5.14（pan/zoom 与撤销栈解耦合同，#38）/ §5.15（空白拖动平移保留选中，单击清选，#39）/ §5.16（标签字号倍率 + 屏幕最小字号钳制，#40）/ §5.17（区域锁定交互与持久化，#41）
- [x] D2 产出 delta `deltas/database/coldrawdb-v1.sql` — migration 0012：area 表加 locked 列（对应 DB 文档增量）
- [x] D3 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 0012 migration 部署说明（启动自动执行先例）
- [x] D4 产出 delta `deltas/test/smoke/` — smoke 增补：区域锁定 → 保存 → reload 持久化断言
- [x] D5 产出 delta `deltas/test/core-KB-shortcut-test-cases.md` — #38：UT-KB-07（pan 后 undo/redo 栈可用性纯函数锚点）+ ST-KB-UNDO-01（e2e：建表 → pan → Ctrl+Z 撤销 → Ctrl+Y 重做）
- [x] D6 产出 delta `deltas/test/core-CR-canvas-test-cases.md` — #39：UT-CR-PAN-01（click/drag 阈值判定纯函数）+ ST-CR-PAN-02（选中表 → 空白拖动平移选中保留 → 单击空白清选）；#40：UT-CR-FONT-01（倍率档位/钳制纯函数）+ ST-CR-FONT-01（e2e 倍率切换 + 低 zoom 钳制探针）；#41：UT-AN-LOCK-01（locked 缺省/序列化兼容）+ ST-AN-03（锁定 e2e：右键锁定 → 拖动无效 → 解锁恢复 → 持久化）
- [x] D7 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [x] C1 #38：ST-KB-UNDO-01 先行复现（**绿，不可复现**——现行构建已含 fix-issues-36-37 的 gloo passive 监听器修复 a3a541a6，根因已被覆盖）→ 交付回归锚点：UT-KB-07（pan/zoom 零触碰撤销栈锚点 + 栈语义纯函数）+ ST-KB-UNDO-01 常绿 + reporter
- [x] C2 #39：空白 pointerdown 不再立即清选，pointerup 位移 <4px（`is_blank_click`）才清选、≥4px pan 保留选中；UT-CR-PAN-01（lib PASS）/ ST-CR-PAN-02（e2e PASS：点表头选中 → pan 保留 → 单击清选 → Shift 框选恢复）+ reporter
- [x] C3 #40：label-font-scale 设置（localStorage + FloatingControls 入口）+ draw 字号钳制；UT-CR-FONT-01 / ST-CR-FONT-01 + reporter；探针 __cdb_lod_probe 增补 label_font_scale / min_font_clamped
- [x] C4 #41：Area.locked 全链路（前端 struct/拖拽门控/右键菜单/Inspector/视觉提示 + 后端 AreaDto + migration 0012）；UT-AN-LOCK-01 / ST-AN-03 + reporter
- [x] C5 smoke 脚本增补区域锁定持久化用例（对齐 D4）

## [deploy] 部署任务

- [ ] E1 verify PASS 且用户授权后：按部署方案重建后端（migration 0012 启动自动执行）+ 前端，确认服务可用
- [ ] E2 用户授权后运行 `openlogos smoke`（区域锁定持久化冒烟须 PASS）

## [follow-up] 收尾

- [ ] F1 verify PASS 后带验收评论关闭 GitHub issue #38 / #39 / #40 / #41
- [ ] F2 smoke PASS 后 `openlogos archive fix-issues-38-41-canvas-interaction`，随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权）
- [x] H2 delta 完成后 `openlogos merge fix-issues-38-41-canvas-interaction`（/goal 授权）
- [ ] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C5 每批闭环），完成后自动提交代码
- [ ] H4 实现完成后 `openlogos verify`（/goal 授权；nice -n 10 + --test-threads=2）
- [ ] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [ ] H6 部署完成后 `openlogos smoke`（/goal 授权）
- [ ] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
