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

## Getting Started

> 当前分支仅保留 **Legacy 后端接口**（`/diagrams`、`/tables`、`/todos`、`/references`、`/templates`）。详细技术栈与架构见 [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)。

### Prerequisites

- Node.js 18+
- npm 9+
- Rust stable (建议通过 `rustup` 安装)
- Cargo

可选：
- SQLite CLI（便于本地查看 `backend/db.sqlite`）

### Local Development

#### 1) 启动后端（Rust + SQLite）

后端默认监听 `127.0.0.1:6666`，配置文件为 `backend/config.toml`。

```bash
cd backend
cargo run
```

首次启动说明：
- `init` 会读取 `backend/config.toml`。
- 若数据库不存在或未初始化，会先执行 `backend/init.sql` 基线建表。
- 然后自动执行 `backend/migrations/*.up.sql`（幂等，版本记录在 `schema_migrations`）。

后端健康检查：

```bash
curl http://127.0.0.1:6666/
# 预期返回: Hello, world!
```

#### 2) 启动前端（Vite + React）

在新的终端窗口中执行：

```bash
git clone https://github.com/drawdb-io/drawdb
cd drawdb
npm install
npm run dev
```

默认访问地址：`http://localhost:5173`

前后端联调说明：
- 前端通过 `vite.config.js` 代理 `/api` 与 `/tables` 等到 `http://localhost:6666`。
- 后端提供 Legacy 接口：`/diagrams`、`/tables`、`/todos`、`/references`、`/templates`。
- 可用 `curl` 或 Postman 访问 `http://127.0.0.1:6666/diagrams/queryAll` 等验证。

### Build

```bash
git clone https://github.com/drawdb-io/drawdb
cd drawdb
npm install
npm run build
```

后端构建：

```bash
cd backend
cargo build
```

### Docker Build

```bash
docker build -t drawdb .
docker run -p 3000:80 drawdb
```

If you wish to work with sharing, set up [server](https://github.com/drawdb-io/drawdb-server) and environment variables according to `.env.sample`. This is not required unless you want to share files.
