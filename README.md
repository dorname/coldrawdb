<div align="center">
    <h1>coldrawdb</h1>
    <p><b>自托管、浏览器端的数据库 ER 图设计工具</b><br>产品理念借鉴 drawDB 与 PDManer，开发中参考 drawDB 源码对齐，核心应用以 Rust 重新实现</p>
    <img width="700" style="border-radius:5px;" alt="coldrawdb 协作编辑器界面（暗色主题）" src="app-editor.png?v=2">
</div>

## 简介

coldrawdb 使用 Rust + Leptos / WASM 构建浏览器编辑器，以 actix-web + SQLite 提供持久化、用户鉴权和实时协作。可以通过 Docker 自托管，也可以从源码运行，并通过本地 MCP 服务让 AI 客户端参与图表编辑。

- **可视化建模**：表、字段、关系、索引、枚举、自定义类型、区域、备注和待办；支持画布与列表视图、关系推断、自动布局、表尺寸调整、关系颜色与线型。
- **导入导出**：多引擎 SQL、DBML、JSON，以及数据字典 Markdown 导出。
- **连接数据库**：读取 PostgreSQL / SQLite 表结构，或向目标数据库执行 DDL；PostgreSQL 支持指定 schema。连接数据库能力与 SQL 文件支持的引擎范围不同。
- **登录与协作**：注册、登录、Token 续期、房间、邀请、成员权限、回收站，以及前后端已接通的 WebSocket OT 实时协作。
- **数据字典**：管理代码映射字典和字典项、绑定字段并持久化。
- **MCP**：11 个工具，支持图表管理、表 / 字段 / 关系编辑、自动布局与房间图协作写入。
- **编辑体验**：暗色模式、命令面板、Monaco 代码视图和撤销 / 重做。

## 快速运行：Docker 部署包

正式部署使用 Docker 镜像与 Compose。Windows / macOS 可使用 Docker Desktop，Linux 使用 Docker Engine + Compose v2。镜像发布工作流配置了 `linux/amd64` 和 `linux/arm64`。

从仓库的 GitHub Releases 页面下载 `coldrawdb-<tag>-deploy.zip`，解压到独立目录。压缩包内容位于根目录，可用系统解压工具，或执行：

```bash
# 将 <tag> 替换为下载的版本
unzip 'coldrawdb-<tag>-deploy.zip' -d coldrawdb-deploy
cd coldrawdb-deploy
docker compose up -d
```

浏览器访问 `http://localhost:9080/`，注册 / 登录后创建房间开始编辑。部署包会拉取对应版本的 GHCR 镜像，无需安装 Rust 或本地编译。停止服务使用 `docker compose down`。

跨机器访问时，在部署目录的 `.env` 中设置实际可访问的前端地址；改变端口时同时更新邀请链接基址：

```dotenv
COLDRAWDB_HTTP_PORT=9080
PUBLIC_BASE_URL=http://192.168.1.10:9080
```

然后重新运行 `docker compose up -d`。`PUBLIC_BASE_URL` 应指向浏览器访问的 SPA 入口。

- `nginx` 默认通过宿主机 9080 端口提供统一入口。
- SQLite 数据位于 `./data/`，日志位于 `./logs/`，备份侧车每日打包数据目录至 `./backups/`。
- 健康检查：`http://localhost:9080/api/v1/diagrams/health`。
- Compose 配置了 `restart: unless-stopped`；健康检查失败本身不会触发容器重启。

源码构建部署：在仓库根目录运行 `docker compose up -d --build`。发布镜像与部署包分别由 [docker.yml](.github/workflows/docker.yml) 和 [release.yml](.github/workflows/release.yml) 生成。

## 从源码运行

### 前置要求

- Rust stable 与 Cargo。
- Trunk：`cargo install --locked trunk`。
- WASM 目标：`rustup target add wasm32-unknown-unknown`。
- 启动脚本使用 Bash、curl；Windows 可使用 WSL2，或直接采用 Docker 部署。
- 应用构建无需 Node.js / npm；Playwright 浏览器测试需要 Node.js / npm 和 Chromium，详见 [贡献指南](CONTRIBUTING.md)。

### 一键启动

在仓库根目录执行：

```bash
./scripts/start-local.sh
# 前端：http://127.0.0.1:8080/editor
# 后端：http://127.0.0.1:3000
./scripts/stop-local.sh
```

脚本先编译后端，再启动后端与 Trunk，自动注入前端 API 地址及邀请链接基址。日志写入 `logs/`。可通过 `COLDRAWDB_BACKEND_PORT` / `COLDRAWDB_FRONTEND_PORT` 调整端口；首次前端编译较慢时可设置 `COLDRAWDB_FRONTEND_TIMEOUT=300`。

### 手动启动

从仓库根目录分别在两个终端执行：

```bash
# 终端一：后端
cd backend
PUBLIC_BASE_URL=http://127.0.0.1:8080 cargo run --release
```

```bash
# 终端二：前端
cd frontend-rs
COLDRAWDB_API_BASE=http://127.0.0.1:3000 trunk serve --port 8080
```

后端读取 `backend/config.toml`；首次启动建立基线表，再执行 `backend/migrations/*.up.sql`，版本记录在 `schema_migrations`。本地默认数据库为 `backend/db.sqlite`。

`COLDRAWDB_API_BASE` 是前端编译期配置，开发时必须与后端地址一致。生产同源部署构建时应取消该变量，使 API 与 WebSocket 使用页面所在域名。

```bash
# 以下命令均从仓库根目录执行
(cd backend && cargo build --release)
(cd frontend-rs && env -u COLDRAWDB_API_BASE trunk build --release --no-wasm-opt)
# 前端产物：frontend-rs/dist/
```

接口检查：

```bash
curl http://127.0.0.1:3000/api/v1/diagrams/health
curl -X POST http://127.0.0.1:3000/api/v1/diagrams \
  -H 'Content-Type: application/json' \
  -d '{"name":"demo","database":"mysql"}'
```

## MCP 接入

`coldrawdb-mcp` 是独立的本地 stdio 服务，仓库提供 Claude、Codex、Cursor、OpenCode 的配置模板。

```bash
./scripts/build-mcp.sh
# 产物：mcp-server/target/release/coldrawdb-mcp
```

配置 `COLDRAWDB_BASE_URL` 为后端可访问的基址，例如本地开发的 `http://127.0.0.1:3000`，或 Compose 入口 `http://localhost:9080`；不要附加 `/api/v1`。

| 类别 | 工具 |
|---|---|
| 读取与导出 | `list_diagrams`、`get_diagram`、`export_schema` |
| 图表写入 | `create_diagram`、`update_diagram`、`delete_diagram`、`import_schema` |
| 画布编辑 | `update_table`、`update_field`、`update_reference`、`layout_diagram` |

`update_diagram` 使用 `expected_revision` 防止覆盖并发修改；删除需要 `confirm=true`。房间图更新收到 `USE_OP_CHANNEL` 后会转为 WebSocket 差异操作，该路径需要有效的 `COLDRAWDB_ACCESS_TOKEN` 与房间写入权限。配置与边界见 [MCP README](mcp-server/README.md)。

## 技术栈与目录

| 路径 | 内容 |
|---|---|
| `frontend-rs/` | Rust + Leptos 0.5 CSR / WASM、Canvas、协作客户端、数据字典、组件与主题；Trunk 构建 |
| `backend/` | actix-web 4 + Tokio、SeaORM 0.12 + SQLite、JWT / Argon2、房间与 OT 协作 |
| `mcp-server/` | Rust stdio MCP 服务，通过后端 HTTP API 与房间 WebSocket 访问图表 |
| `logos/` | 需求、设计、API / 数据库规格、测试定义、变更与验收记录 |
| `scripts/` | 本地服务启动、MCP 构建、验证等脚本 |
| `.github/workflows/` | 构建、测试、镜像与部署包发布工作流 |

前端主体使用 Rust，Monaco 和浏览器测试包含 JavaScript / TypeScript。架构说明见 [架构概览](logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md)。

## API 与实现状态

以下 HTTP 分组挂载于 `/api/v1`，WebSocket 使用独立路径。端点数按当前路由注册统计：

| 分组 | 数量 | 能力 |
|---|---|---|
| diagrams | 6 | 图表创建 / 读取 / 更新 / 删除、健康检查、完整图表导入 |
| bridge | 7 | 配置、本地草稿导入 / 日志 / 重试、数据库连接导入、DDL 执行 |
| auth | 5 | 注册、登录、续期、退出、当前用户 |
| rooms | 13 | 房间、邀请、成员管理、回收站恢复与永久删除 |
| collab | 2 | 房间协作 head / ops |
| WebSocket | 1 | `/ws/rooms/{room_id}`，实时协作 |

兼容路由还包括 `/diagrams/*`、`/tables/*`、`/todos/*`；MCP 列表读取使用 `/diagrams/queryAll`。契约入口为 [`logos/resources/api/`](logos/resources/api/)。

当前代码已覆盖 S01 / S02 编辑保存与分享加载、S03 鉴权、S04 房间管理、S05 前端实时协作、S06 MCP 和 S07 数据字典。资源索引中 S03～S07 的流程状态仍为 `in-progress`；代码实现范围与正式验收状态应分别查看，不能据此视为所有场景已通过发布验收。

## 参与贡献与文档

请阅读 [贡献指南](CONTRIBUTING.md)，其中说明开发环境、测试入口、Issue / PR 要求及 OpenLogos 变更流程。源码变更需要先完成设计与变更提案。

- [资源索引](logos/logos-project.yaml)：现行规格入口。
- [API 编排测试](logos/resources/scenario/)：端到端场景定义。
- [测试结果契约](logos/spec/test-results.md)：OpenLogos reporter 约定。
- [统一交互原型](logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html)：现行 HTML 评审入口。
- [历史重构计划](RUST_WEB_REFACTOR_PLAN.md)：历史背景，当前能力以实现与现行规格为准。

## 致谢与许可证

产品理念与交互设计借鉴 [drawDB](https://github.com/drawdb-io/drawdb) 与 [PDManer 元数建模](https://gitee.com/robergroup/pdmaner)。开发过程中参考 drawDB 源码进行能力对齐，核心应用以 Rust 重新实现；感谢这些项目提供的设计启发。

本项目采用 [MIT 许可证](LICENSE)。
