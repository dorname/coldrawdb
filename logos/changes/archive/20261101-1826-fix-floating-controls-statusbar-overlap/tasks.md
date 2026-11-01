# 实现任务

## [code] 代码实现（纯 CSS 布局修复，无 delta）
- [x] `frontend-rs/src/styles.css` `.cdb-main` 补 `grid-template-rows: minmax(0, 1fr)` 行约束，消除矮视口下画布容器溢出行框
- [x] `frontend-rs/src/styles.css` `.cdb-floating-controls` z-index 10 → 30，防御性保证悬浮条恒可点
- [x] 1280×720 真实栈探针复测：悬浮条全部按钮中心命中按钮本身；1440×900 不回归
- [x] G 批 5 个受影响用例真实通过（无声明式兜底依赖）+ E 批几何用例回归 + 全量验收 Gate 3.6 PASS（run-verify-tests.sh 连续两轮 PASS，含 verify 预跑复跑；G 批 12/12、E 批 6/6；openlogos verify 607/607 执行、0 失败、Gate 3.6 PASS）

## [code] 验收口径附带发现：G 批用例从未真实运行过的修复（服务验收口径）
> 验收中发现 5 个受影响用例的「历史 PASS」均为 0ms 声明式兜底；真实运行暴露测试代码自身缺陷，修复后方可满足「真实通过」口径。产品行为经真实栈探针取证均为符合设计（非产品缺陷）。

- [x] `cr-click-width-rel-style.spec.ts`：4 用例 helper 调用签名错误（`newRoomPage(browser)` 缺 user/roomId 等，13ms 即 TypeError）→ 改为 `registerAndLogin(request,label)` + `createRoomWithDiagram(request,owner,…)` + `newRoomPage(browser,owner,roomId)` 正确签名
- [x] ST-CR-CLICK-01：`tool-area` testid 不存在 → 改 `tool-new-area`；`modal-invite` 常驻 DOM（open 仅切 overlay display），`toHaveCount(0)` 永假 → 改断言 overlay `not.toBeVisible()`
- [x] ST-PB-07：引用不存在的 `refs`/`line_type` debug 字段（真实通道为 `ref_meta`）+ `ALTER TABLE ADD CONSTRAINT` 导入不建关系 → 内联 FOREIGN KEY DDL + data-follow-path 贝塞尔中点选中关系 → Inspector 切 orthogonal → 刷新重进断言保留
- [x] ST-PB-06（cr-comment-color.spec.ts）：像素基线取在选中态（选中/related 态关系线强制高亮色族 fix-31 §4.4，显式关系色仅非选中态渲染）→ 基线移至选中前 + 取消选中后断言像素变化（真实栈探针证实渲染正确）
