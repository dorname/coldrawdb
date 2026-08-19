# 实现任务：对齐统一原型与生产前后端

> module: core | proposal: align-prototype-docs-implementation

## 执行约束

- 先完成本提案确认，再产出 delta；未经用户明确授权不得运行 `openlogos merge align-prototype-docs-implementation`。
- 每个代码批次必须同时包含业务代码、对应 UT/ST/e2e 测试、OpenLogos reporter 写入。
- 输出代码前必须列出本批覆盖的 UT/ST 用例 ID，并与 `logos/resources/test/*.md` 或本变更新增测试说明对齐。
- 不改变已合并的 S03/S04/S05 API、DDL 与产品设计语义；如发现契约冲突，先写明差异与修复方案。

## [delta] 规格变更
- [x] 新增或更新实现状态 delta：将 `core-implementation-checklist.md` 中 V2 生产前端待接入项拆解为 auth、rooms、collab 三批实现任务。
- [x] 新增 S03 编排测试 delta：补齐 `logos/resources/scenario/core-S03-user-auth.json` 的 register → login → me → refresh → logout → refresh 失效链路。
- [x] 更新测试矩阵 delta：补充生产前端接入用例，覆盖 `ST-PU-02`、`ST-PU-04`、`ST-PU-10`～`ST-PU-16` 对应的真实 API/WS 版本。
- [x] 更新验收说明 delta：明确统一主原型仍是视觉与交互基线，历史 S03/S04/S05 原型不作为验收入口。

## [code] 代码实现

- [ ] 实现代码变更