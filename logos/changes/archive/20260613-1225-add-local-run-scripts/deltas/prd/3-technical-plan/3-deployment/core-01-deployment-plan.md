## MODIFIED — 3. 本地 dev 部署

### 3.1 启动顺序

推荐方式（一键脚本）：

```bash
# 1. 从仓库根目录启动前后端
./scripts/start-local.sh

# 2. 访问
open http://localhost:8080/editor
```

手动方式（需要两个终端）：

```bash
# 终端 1：启动后端
cd backend
cargo run --release          # 监听 :3000

# 终端 2：启动前端
cd frontend-rs
trunk serve --port 8080      # 监听 :8080
```

### 3.2 数据初始化

- 首次启动后端时自动运行 `backend/init.sql`（含 11 张表 DDL）
- 配置文件：`backend/config.toml`

```toml
# backend/config.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite://data/coldrawdb.db?mode=rwc"

[logging]
level = "info"
format = "json"
```

### 3.3 依赖

| 工具 | 最低版本 | 用途 |
|---|---|---|
| Rust | 1.75 | 编译前后端 |
| trunk | 0.20 | 打包 WASM |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| SQLite | 3.40+ | WAL 模式（默认启用） |
| Node.js | (非必须) | trunk 内部用 |

### 3.4 使用本地启动脚本

为简化本地开发，仓库根目录提供 `scripts/start-local.sh` 和 `scripts/stop-local.sh`。

#### 3.4.1 脚本位置

```
scripts/
├── start-local.sh    # 启动后端 + 前端
├── stop-local.sh     # 停止由 start-local.sh 启动的进程
└── common.sh         # 公共函数（日志、PID、端口检测）
```

#### 3.4.2 启动脚本行为

`start-local.sh` 默认行为：

1. 检查依赖：`cargo`、`trunk`、Rust wasm32 target 是否可用
2. 检查端口占用：后端默认 `3000`，前端默认 `8080`
3. 创建 `logs/` 目录
4. 启动后端：`cd backend && cargo run --release`，输出重定向到 `logs/backend.log`
5. 等待后端健康检查通过（`GET /api/v1/diagrams/health` 或 `GET /`）
6. 启动前端：`cd frontend-rs && trunk serve --port 8080`，输出重定向到 `logs/frontend.log`
7. 将前后端 PID 写入 `logs/backend.pid` 与 `logs/frontend.pid`
8. 输出访问地址与停止命令

启动示例：

```bash
./scripts/start-local.sh
# 或自定义端口
COLDRAWDB_BACKEND_PORT=3001 COLDRAWDB_FRONTEND_PORT=8081 ./scripts/start-local.sh
```

#### 3.4.3 停止脚本行为

`stop-local.sh` 默认行为：

1. 读取 `logs/backend.pid` 与 `logs/frontend.pid`
2. 先向前端进程发送 `TERM` 信号
3. 再向后端进程发送 `TERM` 信号
4. 等待最多 10 秒，进程仍未退出则发送 `KILL`
5. 删除 PID 文件

停止示例：

```bash
./scripts/stop-local.sh
```

#### 3.4.4 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `COLDRAWDB_BACKEND_PORT` | `3000` | 后端监听端口 |
| `COLDRAWDB_FRONTEND_PORT` | `8080` | 前端 trunk serve 端口 |
| `COLDRAWDB_BACKEND_LOG` | `logs/backend.log` | 后端日志路径（相对仓库根目录） |
| `COLDRAWDB_FRONTEND_LOG` | `logs/frontend.log` | 前端日志路径 |
| `COLDRAWDB_BACKEND_PID` | `logs/backend.pid` | 后端 PID 文件路径 |
| `COLDRAWDB_FRONTEND_PID` | `logs/frontend.pid` | 前端 PID 文件路径 |
| `COLDRAWDB_HEALTH_TIMEOUT` | `60` | 等待后端健康检查超时秒数 |

#### 3.4.5 日志与 PID

- 所有脚本日志统一输出到 `logs/` 目录
- 运行产生的 `logs/`、`*.pid` 已加入 `.gitignore`，不会被提交
- 如需排查启动失败，查看 `logs/backend.log` 与 `logs/frontend.log`
