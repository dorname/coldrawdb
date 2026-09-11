# Delta — core-smoke-test-cases.md（修改）

> module: core | proposal: harden-local-dev-cold-start

## MODIFIED — 8.2 步骤

### 8.2 步骤

1. 确保本地无占用 3000 / 8080 端口的进程
2. 从仓库根目录执行 `./scripts/start-local.sh`（脚本会先 `cargo build --release` 预编译后端再启动）
3. 等待脚本输出 "Services started successfully"
4. 验证 `logs/backend.pid` 与 `logs/frontend.pid` 存在且对应进程存活
5. `curl http://127.0.0.1:3000/` → 期望 200，body 含 `Hello, world!`
6. `curl http://127.0.0.1:8080/` → 期望 200，body 含 `<div id="app">`
7. 执行 `./scripts/stop-local.sh`
8. 验证前后端 PID 文件被删除
9. 等待 5 秒后 `curl` 前后端地址 → 期望连接失败

## MODIFIED — 8.3 断言

### 8.3 断言

- `start-local.sh` 退出码为 0
- 后端健康检查在 120 秒内通过
- 前端 HTTP 入口在 30 秒内可达
- `stop-local.sh` 退出码为 0
- 停止后 3000 / 8080 端口无监听进程

## MODIFIED — 8.4 失败处理

### 8.4 失败处理

- 端口冲突 → 检查并释放占用进程；后端端口固定为 `backend/config.toml` 的 3000（`COLDRAWDB_BACKEND_PORT` 仅改变脚本侧检查地址，不能用于规避冲突），前端可用 `COLDRAWDB_FRONTEND_PORT` 调整
- 后端启动失败 → 查看 `logs/backend.log`（冷机首次 `cargo build --release` 耗时较长，若日志显示仍在编译，增大 `COLDRAWDB_HEALTH_TIMEOUT` 重试）
- 前端启动失败 → 查看 `logs/frontend.log`
- 停止失败 → 手动 `kill -9` 对应 PID 后排查脚本

## ADDED — 附录 A：用例 ID 清单

### 附录 A：用例 ID 清单

| ID | 标题 |
|---|---|
| SMOKE-core-01 | 健康检查 |
| SMOKE-core-02 | 创建 + 读取 E2E |
| SMOKE-core-03 | 导入导出 E2E |
| SMOKE-core-04 | 静态资源加载 |
| SMOKE-core-05 | 数据库健康 |
| SMOKE-core-06 | 本地脚本启停验证 |
