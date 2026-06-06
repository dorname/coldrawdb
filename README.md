<div align="center">
  <sup>Special thanks to:</sup>
  <br>
  <a href="https://www.warp.dev/drawdb/" target="_blank">
    <img alt="Warp sponsorship" width="280" src="https://github.com/user-attachments/assets/c7f141e7-9751-407d-bb0e-d6f2c487b34f">
    <br>
    <b>Next-gen AI-powered intelligent terminal for all platforms</b>
  </a>
</div>

<br/>
<br/>

<div align="center">
    <img width="64" alt="drawdb logo" src="./src/assets/icon-dark.png">
    <h1>drawDB</h1>
</div>

<h3 align="center">Free, simple, and intuitive database schema editor and SQL generator.</h3>

<div align="center" style="margin-bottom:12px;">
    <a href="https://drawdb.app/" style="display: flex; align-items: center;">
        <img src="https://img.shields.io/badge/Start%20building-grey" alt="drawDB"/>
    </a>
    <a href="https://discord.gg/BrjZgNrmR6" style="display: flex; align-items: center;">
        <img src="https://img.shields.io/discord/1196658537208758412.svg?label=Join%20the%20Discord&logo=discord" alt="Discord"/>
    </a>
    <a href="https://x.com/drawDB_" style="display: flex; align-items: center;">
        <img src="https://img.shields.io/badge/Follow%20us%20on%20X-blue?logo=X" alt="Follow us on X"/>
    </a>
</div>

<h3 align="center"><img width="700" style="border-radius:5px;" alt="demo" src="drawdb.png"></h3>

DrawDB is a robust and user-friendly database entity relationship (DBER) editor right in your browser. Build diagrams with a few clicks, export sql scripts, customize your editor, and more without creating an account. See the full set of features [here](https://drawdb.app/).

## Tech Stack

> **Phase 4 起**前端从 React 完全替换为 Rust Web（WASM）。

| Layer | Tech |
|-------|------|
| Frontend | **Rust + Leptos 0.x + WASM**（`frontend-rs/` crate，1 crate + 4 modules） |
| Bundler | `trunk`（WASM bundler） |
| State | Leptos signals / `create_store` 细粒度响应式 |
| Rendering | HTML5 `<canvas>` + 贝塞尔连线（自渲染，无 vDOM diff） |
| Backend | **Rust + actix-web 4**（`backend/`，端口 `127.0.0.1:6666`） |
| Persistence | SQLite（`backend/db.sqlite`）+ `sqlx` 迁移 |
| API | REST v1（`/api/v1/diagrams/*`），含 409 revision 冲突语义 |
| E2E | `wasm-pack test --chrome` + Playwright（CI 强制） |
| CI | GitHub Actions：`cargo build --release` + `trunk build` + `mmdc` 渲染 + ast-grep module gate |

架构图：[`docs/phase4/architecture.mmd`](docs/phase4/architecture.mmd)（4 modules + 单向依赖）。

## Getting Started

> **Phase 4 完成**：React 前端已**完全下线**并替换为 Rust Web（WASM + Leptos）。
> 后端保持 Rust + actix-web + SQLite；前端重建为 `frontend-rs/` crate。
> 里程碑总览见 `docs/MILESTONE_V1_INITIAL.md`；Phase 4 收官报告见 `docs/phase4/PHASE4_DONE.md`。

### Prerequisites

- Rust stable (建议通过 `rustup` 安装)
- Cargo
- `trunk`（WASM bundler：`cargo install --locked trunk`）
- 可选：`wasm-pack`（集成测试用）
- 可选：Playwright + Chromium（E2E + perf 测量用）

不需要 Node.js / npm — Phase 4 起所有前端构建走 Rust 工具链。

### Local Development

#### 1) 启动后端（Rust + SQLite）

后端默认监听 `127.0.0.1:6666`，配置文件为 `backend/config.toml`。

```bash
cd backend
cargo run --release
```

> 性能与 §8 指标测量**必须**使用 release 模式（plan W3-2）；debug build 跑 P95 不可信。

首次启动说明：
- `init` 会读取 `backend/config.toml`。
- 若数据库不存在或未初始化，会先执行 `backend/init.sql` 基线建表。
- 然后自动执行 `backend/migrations/*.up.sql`（幂等，版本记录在 `schema_migrations`）。

后端健康检查：

```bash
curl http://127.0.0.1:6666/
# 预期返回: Hello, world!
```

#### 2) 启动前端（Rust Web + Leptos + WASM + trunk）

在新的终端窗口中执行：

```bash
cd frontend-rs
trunk serve --port 8080
```

默认访问地址：`http://localhost:8080`

前后端联调说明（Phase 4 起）：
- **无前端代理**：`vite.config.js` 已删除；frontend-rs 通过 `fetch` 直连 `127.0.0.1:6666`。
  CORS 由后端 `actix-cors` 配置（dev 环境全开）。
- 后端核心接口位于 `/api/v1/*`（diagrams v1 CRUD + 409 revision 冲突语义）。
- 数据流：`editor-data-access` → `editor-core` (debounce 1s) → `editor-panels` /
  `editor-render`（Leptos signals 细粒度更新）。

#### 3) 常用后端接口快速验证

```bash
# 创建 diagram
curl -X POST http://127.0.0.1:6666/api/v1/diagrams \
  -H 'Content-Type: application/json' \
  -d '{"name":"demo","engine":"mysql"}'

# 查询 bridge 配置
curl http://127.0.0.1:6666/api/v1/bridge/config
```

### Build

后端构建（release）：

```bash
cd backend
cargo build --release
```

前端构建（trunk release）：

```bash
cd frontend-rs
trunk build --release
# 产物：frontend-rs/dist/index.html + pkg/frontend_rs.wasm + pkg/frontend_rs.js
```

### Docker Build

```bash
docker build -t drawdb .
docker run -p 3000:80 drawdb
```

If you wish to work with sharing, set up [server](https://github.com/drawdb-io/drawdb-server) and environment variables according to `.env.sample`. This is not required unless you want to share files.
