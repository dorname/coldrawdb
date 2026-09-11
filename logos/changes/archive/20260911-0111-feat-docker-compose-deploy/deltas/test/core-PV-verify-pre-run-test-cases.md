# Delta: core-PV-verify-pre-run-test-cases.md

> 提案：feat-docker-compose-deploy

## MODIFIED — 1. 新增用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-PU-20 | 工作区不存在 `backend/test.sqlite` | 执行关联查询单测 | 使用唯一临时 SQLite 文件并初始化 schema；查询成功；不创建工作区数据库 |
| ST-PU-20 | 已存在一份正式测试账本 | 执行 `scripts/run-verify-tests.sh` | 后端、前端、ST-PU-19 与账本校验全部通过；失败时恢复旧账本；成功时只保留完整新账本 |
| UT-PU-21 | — | 锚点检查 `backend/src/main.rs` 与 `scripts/start-local.sh` | `init_log` 含 `with_ansi(false)` 且不含 `.pretty()`；默认 filter 含 `sqlx::query=warn`；start-local.sh 的 trunk 启动含 `NO_COLOR=true`（trunk 仅接受 bool，fix-local-start-and-invite-config 修订）且不再 `-u NO_COLOR` |
| UT-DP-01 | — | 调用环境变量覆盖纯函数 | `parse_bind_addr` 解析 `host:port`（IPv4/IPv6/缺省拒绝）；`db_path_from_url` 剥离 `sqlite://` 前缀与 query；`apply_env_overrides` 按 `COLDRAWDB_BIND_ADDR` / `COLDRAWDB_DB_URL` 覆盖 Config 且不回写 config.toml |
| UT-DP-02 | — | 调用 `cache_control_for_path` | `.wasm`/`.js` → `public, max-age=31536000, immutable`；`/`、`index.html`、无扩展名前端路由 → `no-cache`；`/api/`、`/ws/` 路径 → 不加缓存头 |
| UT-DP-03 | 无 DB 依赖 | `GET /api/v1/diagrams/health` | 200 + body `{"status":"ok"}`；静态段 `health` 优先于动态段 `{id}` |
| UT-DP-04 | 临时静态目录含 index.html / app.wasm | 经 configure_static(Some) 挂载后请求 | `GET /` 与 `GET /editor`（SPA 回源）返回 index.html 且 `Cache-Control: no-cache`；`GET /app.wasm` 返回 200 且 `Cache-Control: public, max-age=31536000, immutable` |
| UT-DP-05 | 静态目录不存在 | 经 configure_static(None) 挂载后请求 | `GET /` 返回 200 存活文案（纯 API 模式，不 panic）；未知路径返回 404 |

## ADDED — 3. feat-docker-compose-deploy 用例详情

### UT-DP-01 — 环境变量覆盖纯函数（init.rs）

- **位置**：`backend/src/init.rs` 测试模块
- **断言**：
  1. `parse_bind_addr("0.0.0.0:3000")` → `Some(("0.0.0.0", 3000))`；`"[::]:8080"` 等非法端口 / 无冒号 → `None`
  2. `db_path_from_url("sqlite:///data/coldrawdb.db?mode=rwc")` → `/data/coldrawdb.db`；`"sqlite://data/db.sqlite"` → `data/db.sqlite`
  3. `apply_env_overrides` 仅修改传入的 `Config` 副本（server.host/port、database.path），不回写 config.toml 文件

### UT-DP-02 — 缓存头决策纯函数（static_serve.rs）

- **位置**：`backend/src/static_serve.rs` 测试模块
- **断言**：`.wasm` / `.js` → immutable 一年；`/` / `index.html` / `/editor` 等无扩展名路径 → `no-cache`；`/api/*` / `/ws/*` → `None`（不干预 API 响应）

### UT-DP-03 — health 端点（diagrams_v1.rs）

- **位置**：`backend/src/diagrams_v1.rs` 测试模块
- **断言**：`GET /api/v1/diagrams/health` → 200，body JSON `{"status":"ok"}`；不被 `/diagrams/{id}` 动态路由吞掉

### UT-DP-04 — 静态服务 + SPA 回源 + 缓存头（static_serve.rs）

- **位置**：`backend/src/static_serve.rs` 测试模块（actix_web::test + 唯一临时目录）
- **断言**：`GET /` → 200 index.html 内容 + `no-cache`；`GET /editor` → 200 同一 index.html（SPA 回源）+ `no-cache`；`GET /app.wasm` → 200 + immutable 缓存头；测试结束删除临时目录

### UT-DP-05 — 纯 API 模式降级（static_serve.rs）

- **位置**：`backend/src/static_serve.rs` 测试模块
- **断言**：`configure_static(cfg, None)` 下 `GET /` → 200 存活文案（含 `coldrawdb`）；`GET /definitely-missing` → 404；不 panic
