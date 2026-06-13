# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 更新本地 dev 部署章节，添加 start/stop 脚本说明
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — 新增 SMOKE-core-06 本地脚本启停验证用例

## [code] 代码实现

- [x] 新增 `scripts/start-local.sh` — 启动后端（`cargo run --release`）和前端（`trunk serve --port 8080`），写入 PID 与日志
- [x] 新增 `scripts/stop-local.sh` — 读取 PID 文件并安全停止前后端进程
- [x] 新增 `scripts/common.sh` — 公共函数：日志目录初始化、端口检测、PID 管理、错误处理
- [x] 为脚本添加可执行权限，并添加最小 smoke/集成测试代码验证启停链路
- [x] 更新 `.gitignore` 忽略本地运行产生的 `logs/` 和 `*.pid`
