# 实现任务

> module: core | proposal: optimize-prototype-dark-glass-contrast

## [delta] 原型与规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：全页面暗色色板、玻璃态面板、字体对比度优化
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`：更新 auth 页色彩/字体/对比度说明
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`：更新 rooms 页色彩/字体/对比度说明
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`：更新 editor 页色彩/字体/对比度说明

## [review] 艺术总监审核
- [x] 第 1 轮：提交调整后的原型与修改说明，等待总监反馈（2026-08-21 提交；艺术总监代理实测 verdict：需调整（轻度），2 项 P0）
- [x] 第 2 轮：落实 P0 修复（.remote-label 深字覆盖 + invite h1 `<br>` 断行）并回交复审，直到总监回复「审核通过」（2026-08-21 复审通过：remote-label 实测 7.49:1 达 AAA，invite 两态断行正常，四视图回归零异常）

## [merge] 规格合并
- [x] 审核通过后，提醒用户明确授权执行 `openlogos merge optimize-prototype-dark-glass-contrast`（2026-08-21 已授权并生成 MERGE_PROMPT）
- [x] 按 MERGE_PROMPT 合并 delta 到 `logos/resources/`（S03/S04/S05 视觉基准行 + dark 主题验收行已落盘；HTML 全量 delta 与资源一致为 no-op；SPEC_MERGED 已写入）
- [ ] AI 自动 commit 合并后的规格文档

## [verify] 轻量验收
- [ ] 确认合并后的原型文件可在浏览器正常打开，无样式崩溃或横向溢出
- [ ] 提醒用户明确授权执行 `openlogos verify`（本提案无新增 UT/ST，主要验证规格完整性与原型可访问）

## [archive] 归档
- [ ] `verify` 通过后，提醒用户明确授权执行 `openlogos archive optimize-prototype-dark-glass-contrast`
- [ ] AI 自动 commit 归档
- [ ] 提醒用户确认是否执行 `git push`
