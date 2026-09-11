# core-PV 验收预跑与 reporter 账本用例

> module: core | proposal: improve-unified-collab-prototype | type: verify infrastructure

## 1. 新增用例

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

## 2. 既有标题式用例兼容索引

以下用例已有详细规格与真实 reporter，但旧文档只以 Markdown 标题声明。此表为 OpenLogos CLI 提供可解析索引，不改变原用例语义。

| ID | 详细规格 | reporter 来源 |
|---|---|---|
| UT-S03-01 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-02 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-03 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-04 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-05 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-06 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-07 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| ST-S03-01 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S04-01 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-02 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-03 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-04 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-05 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-06 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-07 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-08 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-09 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-10 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-11 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-12 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-13 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-14 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| ST-S04-01 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-C-01 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-02 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-03 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-04 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-05 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-06 | `core-S05-test-cases.md` | `backend/src/diagrams_v1.rs` |
| UT-C-07 | `core-S05-test-cases.md` | `backend/src/collab/mod.rs` |
| UT-C-08 | `core-S05-test-cases.md` | `backend/src/diagrams_v1.rs` |
| UT-C-09 | `core-S05-test-cases.md` | `backend/src/diagrams_v1.rs` |
| UT-C-10 | `core-S05-test-cases.md` | `backend/src/collab/mod.rs` |
| ST-C-01 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| ST-B-01 | `core-PC-import-export-test-cases.md` | `backend/src/phase3_bridge.rs` |
| UT-ALIGN-B01 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-ALIGN-B02 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-ALIGN-B03 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-01 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-02 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-03 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-S04-UI-12 | `core-S04-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-S04-UI-13 | `core-S04-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |

## 3. 验收约束

- 正式账本只允许在完整预跑成功后替换；任一阶段失败必须恢复运行前内容。
- 自动化 reporter 的 ID 必须全部出现在可解析用例表中，且所有非 `[manual]` 用例必须有最终结果。
- ST-PU-01～18 属于人工视觉与完整交互验收，不写 JSONL；ST-PU-19、ST-PU-20 必须写入 JSONL。

## 4. 本次实测结果（2026-08-18）

| 用例 | 结果 | 实测证据 |
|---|---|---|
| UT-PU-20 | PASS | 唯一临时 SQLite 文件完成 schema 初始化与空关联查询，测试结束后删除临时文件 |
| ST-PU-20 失败恢复 | PASS | 使用失败 cargo 可执行文件触发预跑中止；运行前后账本均为 35 行且 SHA-256 保持 `9ca340b9735bc750da37a5d897cfc2c65cc3aaf87a0b00d05094c11dba86252f` |
| ST-PU-20 完整预跑 | PASS | 后端 43/43 + bootstrap 1/1；前端 Rust 135 项；原型 ST-PU-19；账本 133 个定义、18 个人工、115/115 自动化结果 |

### UT-PU-21 — 日志格式锚点（后端 ANSI/格式 + 前端启动脚本）

- **位置**：`backend/src/main.rs` 测试模块（读自身源码与 `../scripts/start-local.sh`）
- **断言**：
  1. `init_log` 含 `.with_ansi(false)`；不含 `.pretty()`
  2. 默认 EnvFilter 含 `sqlx::query=warn`（降噪；RUST_LOG 可覆盖）
  3. `start-local.sh` trunk 启动行含 `NO_COLOR=true`（trunk 仅接受 bool，fix-local-start-and-invite-config 修订）且不含 `-u NO_COLOR`

## 5. feat-docker-compose-deploy 用例详情

### UT-DP-01 — 环境变量覆盖纯函数（init.rs）

- **位置**：`backend/src/init.rs` 测试模块
- **断言**：
  1. `parse_bind_addr("0.0.0.0:3000")` → `Some(("0.0.0.0", 3000))`；非法端口 / 无冒号 → `None`
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
