# 实现任务：生产前端继续对齐统一主原型

> module: core | proposal: align-frontend-to-prototype

## 执行约束

- 本提案延续 `core-01-editor-prototype.html` 作为唯一现行视觉与交互基线；历史 S03/S04/S05 独立原型仅作差异参考。
- 不改变已合并的 auth / rooms / collab API、DDL 与后端语义；如发现契约冲突，先写明差异与修复方案。
- 每个代码批次必须同时包含业务代码、对应 UT/ST/e2e 或 reporter 覆盖、OpenLogos reporter 写入。
- 输出代码前必须列出本批覆盖的 UT/ST 用例 ID，并与 `logos/resources/test/*.md` 或本变更新增测试说明对齐。
- `?share=` 匿名只读链路、S01 保存/409、IO 抽屉、命令面板和设计系统不可回退。

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/reference/implementation/core-frontend-alignment-acceptance.md`：新增生产前端对齐主原型的页面流验收标准，区分“API 已接入”和“体验已对齐”。
- [x] 产出 delta 文件到 `deltas/test/core-V2-production-frontend-test-cases.md`：新增 `ST-FE-PROTO-01`～`ST-FE-PROTO-08`，覆盖 auth、rooms、invite、collab editor 与 720px 响应式。
- [x] 产出 delta 文件到 `deltas/reference/implementation/core-implementation-checklist.md`：新增 7.6 生产前端原型对齐收口项，记录本提案 A/B/C/D 批次。
- [x] 产出 delta 文件到 `deltas/reference/project/core-delta-index.md`：说明本次不新增 API/DB/场景 JSON，仅补生产前端体验和 reporter。

## [code] 代码实现

- [ ] 实现代码变更

## [deploy] 部署与冒烟

- [ ] 本变更需要部署，原因与 `proposal.md` 的“部署影响”一致：前端页面流、路由与静态资源变化。
- [ ] 部署执行是人工确认点；用户已授权，执行时仍记录命令与结果。
- [ ] smoke 是人工确认点；部署完成后运行 `openlogos smoke --env staging` 并提交报告。
