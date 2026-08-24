# Delta — core-S02-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

S02：分享链接匿名只读加载。与统一页面流关系：`?share=` **旁路** auth/rooms，不被鉴权阻断；无 share 参数时走 auth→rooms。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（A 批）将分享只读与路由用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — ADDED / MODIFIED — 分享只读与路由

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S02-SHARE-RO | 未登录 | 打开 `?share=<valid>` | 进入只读编辑器；写工具禁用；不强制登录 | 本提案 A 批实现 |
| ST-S02-404 | 未登录 | `?share=<missing>` 或 GET 404 | 友好 404；不泄漏内部错误；不枚举私有房间 | 本提案 A 批实现 |
| ST-S02-NO-SHARE | 未登录 | 打开默认入口（无 share） | 进入 **auth**；不进入私有 rooms 数据 | 本提案 A 批实现 |
| ST-S02-SHARE-VS-AUTH | 已登录 | 打开 `?share=` | 仍只读分享态；不因已登录自动升级为可写 | 本提案 A 批实现 |
| UT-S02-ROUTE-01 | URL 解析 | 同时有 diagram path 与 `?share=` | share 优先；`share_mode=true` | 本提案 A 批实现 |
