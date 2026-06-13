## ADDED — 8. SMOKE-core-06 — 本地脚本启停验证

### 8.1 目的

验证 `scripts/start-local.sh` 能正确拉起前后端服务，`scripts/stop-local.sh` 能安全停止服务。

### 8.2 步骤

1. 确保本地无占用 3000 / 8080 端口的进程
2. 从仓库根目录执行 `./scripts/start-local.sh`
3. 等待脚本输出 "Services started successfully"
4. 验证 `logs/backend.pid` 与 `logs/frontend.pid` 存在且对应进程存活
5. `curl http://127.0.0.1:3000/` → 期望 200，body 含 `Hello, world!`
6. `curl http://127.0.0.1:8080/` → 期望 200，body 含 `<div id="root">`
7. 执行 `./scripts/stop-local.sh`
8. 验证前后端 PID 文件被删除
9. 等待 5 秒后 `curl` 前后端地址 → 期望连接失败

### 8.3 断言

- `start-local.sh` 退出码为 0
- 后端健康检查在 60 秒内通过
- 前端 HTTP 入口在 30 秒内可达
- `stop-local.sh` 退出码为 0
- 停止后 3000 / 8080 端口无监听进程

### 8.4 失败处理

- 端口冲突 → 提示用户检查占用进程或修改 `COLDRAWDB_BACKEND_PORT` / `COLDRAWDB_FRONTEND_PORT`
- 后端启动失败 → 查看 `logs/backend.log`
- 前端启动失败 → 查看 `logs/frontend.log`
- 停止失败 → 手动 `kill -9` 对应 PID 后排查脚本
