# 实现任务

> module: core | proposal: fix-backend-v1-scope-routing

## [delta] smoke 测试补充
- [ ] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md`：新增 `SMOKE-core-07`，覆盖 `/api/v1/auth/register` 返回 201、`/api/v1/auth/login` 返回 200，确保 V2 auth 路由可达

## [code] 代码实现

- [ ] 实现代码变更

## [deploy] 部署任务
- [ ] 停止当前运行的后端进程
- [ ] 启动新编译的后端（本地 staging 等价环境）
- [ ] 运行 `openlogos smoke --env staging` 验证 `SMOKE-core-07` 通过
