# 实现任务

> module: core | proposal: polish-dark-mode-micro-contrast

## [delta] 原型与规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：avatar initials 深色字、field-type/constraint 升 10px、invite h1 品牌色 span
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`：补充头像对比度与 invite 标题强调说明
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`：补充 presence 头像对比度与 10px 字号地板说明

## [review] 艺术总监审核
- [x] 第 1 轮：提交调整后的原型与修改说明，艺术总监代理真渲染实测（含「initials × 各成员色背景」组合矩阵），等待反馈（2026-08-21 提交；2026-08-22 verdict 审核通过，附 3 项裁决）
- [x] 第 2 轮：按裁决 C 落实 light 侧 avatar 规则并回交定向复测（2026-08-22 通过：light 内联 9 组合 5.53–7.76:1、渐变头像零误伤、dark 零回归）

## [merge] 规格合并
- [x] 审核通过后，提醒用户明确授权执行 `openlogos merge polish-dark-mode-micro-contrast`（2026-08-22 已授权并生成 MERGE_PROMPT）
- [x] 按 MERGE_PROMPT 合并 delta 到 `logos/resources/`（原型全量覆盖；S04/S05 视觉基准行与 dark 验收行已落盘并读回确认；SPEC_MERGED 已写入）
- [x] AI 自动 commit 合并后的规格文档（7b7158b `docs(polish-dark-mode-micro-contrast): merge spec deltas`）

## [verify] 轻量验收
- [x] 确认合并后的原型文件可在浏览器正常打开，无样式崩溃或横向溢出（艺术总监两轮回归扫描覆盖：双模式八组视图零溢出、零 JS 错误、零错位）
- [x] 提醒用户明确授权执行 `openlogos verify`（本提案无新增 UT/ST，主要验证规格完整性与原型可访问）（2026-08-22 已授权执行：211 用例 100% 覆盖、0 失败、Gate 3.6 PASS，VERIFY_PASS 已写入）

## [archive] 归档
- [x] `verify` 通过后，提醒用户明确授权执行 `openlogos archive polish-dark-mode-micro-contrast`（2026-08-22 已授权执行，guard 已删除）
- [x] AI 自动 commit 归档
- [ ] 提醒用户确认是否执行 `git push`
