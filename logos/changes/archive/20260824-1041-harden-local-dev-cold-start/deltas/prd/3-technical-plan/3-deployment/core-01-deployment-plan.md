# Delta — core-01-deployment-plan.md（修改）

> module: core | proposal: harden-local-dev-cold-start

## MODIFIED — 3.4.2 启动脚本行为

#### 3.4.2 启动脚本行为

`start-local.sh` 默认行为：

1. 检查依赖：`cargo`、`trunk`、Rust wasm32 target 是否可用
2. 检查端口占用：后端默认 `3000`，前端默认 `8080`
3. 创建 `logs/` 目录
4. 预编译后端：`cd backend && cargo build --release`，输出重定向到 `logs/backend.log`（避免冷机首次编译挤占健康检查窗口）
5. 启动后端：`cd backend && ./target/release/backend`，输出重定向到 `logs/backend.log`
6. 等待后端健康检查通过（`GET /api/v1/diagrams/health` 或 `GET /`）；若超时且日志显示仍在编译，脚本给出针对性提示
7. 启动前端：`cd frontend-rs && trunk serve --port 8080`，输出重定向到 `logs/frontend.log`
8. 将前后端 PID 写入 `logs/backend.pid` 与 `logs/frontend.pid`
9. 输出访问地址与停止命令

启动示例：

```bash
./scripts/start-local.sh
# 自定义前端端口（后端端口固定为 backend/config.toml 的 3000）
COLDRAWDB_FRONTEND_PORT=8081 ./scripts/start-local.sh
```

> 注意：`COLDRAWDB_BACKEND_PORT` 仅影响脚本侧的端口占用检查与健康检查地址，不改变后端实际绑定端口（固定由 `backend/config.toml` 的 `server.port = 3000` 决定）。显式将其设为非 3000 会导致健康检查轮询错误端口。

## MODIFIED — 3.4.4 环境变量

#### 3.4.4 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `COLDRAWDB_BACKEND_PORT` | `3000` | 脚本侧端口占用检查与健康检查地址所用端口；后端实际绑定端口固定由 `backend/config.toml` 决定（3000） |
| `COLDRAWDB_FRONTEND_PORT` | `8080` | 前端 trunk serve 端口 |
| `COLDRAWDB_BACKEND_LOG` | `logs/backend.log` | 后端日志路径（相对仓库根目录） |
| `COLDRAWDB_FRONTEND_LOG` | `logs/frontend.log` | 前端日志路径 |
| `COLDRAWDB_BACKEND_PID` | `logs/backend.pid` | 后端 PID 文件路径 |
| `COLDRAWDB_FRONTEND_PID` | `logs/frontend.pid` | 前端 PID 文件路径 |
| `COLDRAWDB_HEALTH_TIMEOUT` | `120` | 等待后端健康检查超时秒数 |
