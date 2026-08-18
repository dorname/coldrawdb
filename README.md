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
| Frontend | **Rust + Leptos 0.5 + WASM**（`frontend-rs/` crate，4 modules：data_access / core / panels / render） |
| Bundler | `trunk`（WASM bundler） |
| State | Leptos signals / `create_store` 细粒度响应式（`features=["csr"]`） |
| Rendering | HTML5 `<canvas>` + 贝塞尔连线（自渲染，无 vDOM diff） |
| Design System | `--cdb-*` 设计 token 体系（13 类 ~100 个）+ SVG 图标库 + 8 类核心组件（redesign-phase-e E1–E3） |
| Layout (V2) | AppBar + ToolRail + Inspector + ModalRoot + IO Drawer（6 层 z-index 体系，redesign-phase-a/b/c） |
| Code Editor | Monaco Editor + DBML setup + 复制按钮（E4 替代 V1 `<textarea readonly>`） |
| Theme | Light / Dark 全局切换（`core-0b-dark-mode.md`） |
| Backend | **Rust + actix-web 4**（`backend/`，端口 `127.0.0.1:3000`） |
| Persistence | SQLite（`backend/db.sqlite`，11 张表，WAL 模式）+ SeaORM |
| API | REST v1（`/api/v1/diagrams/*` 5 端点 + `/api/v1/bridge/*` 5 端点），含 409 revision 冲突语义 |
| E2E | `wasm-pack test --chrome` + Playwright（CI 强制） |
| CI | GitHub Actions：`cargo build --release` + `trunk build` + `mmdc` 渲染 + ast-grep module gate |

架构图：[`docs/phase4/architecture.mmd`](docs/phase4/architecture.mmd)（4 modules + 单向依赖）；V2 布局与设计系统详见 `logos/resources/reference/core-baseline-reference.md`。

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

后端默认监听 `127.0.0.1:3000`，配置文件为 `backend/config.toml`。

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
curl http://127.0.0.1:3000/
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
- **无前端代理**：`vite.config.js` 已删除；frontend-rs 通过 `fetch` 直连 `127.0.0.1:3000`。
  CORS 由后端 `actix-cors` 配置（dev 环境全开）。
- 后端核心接口位于 `/api/v1/*`（diagrams v1 CRUD + 409 revision 冲突语义）。
- 数据流：`editor-data-access` → `editor-core` (debounce 1s) → `editor-panels` /
  `editor-render`（Leptos signals 细粒度更新）。

#### 3) 常用后端接口快速验证

```bash
# 创建 diagram
curl -X POST http://127.0.0.1:3000/api/v1/diagrams \
  -H 'Content-Type: application/json' \
  -d '{"name":"demo","engine":"mysql"}'

# 查询 bridge 配置
curl http://127.0.0.1:3000/api/v1/bridge/config
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

## Project Status & Recent Archives

> **当前状态**：coldrawdb 处于 `core` 模块 `launched` 生命周期；活跃变更 `align-unified-prototype-and-add-mcp` 已完成规格合并，代码分批实现中。

现行 HTML 评审入口只有 `core-01-editor-prototype.html`。S01/S02 生产前后端已实现；S03/S04/S05 的 auth、rooms、collab REST/DB/WS 与测试已实现，生产前端登录、房间和 WS/OT/presence 尚未接入。

### MCP（规划/实现中）

S06 计划通过本地 stdio MCP 服务支持 Claude、Codex、Cursor 和 OpenCode，MVP 提供 7 个图表 CRUD/导入/导出工具，不包含 Streamable HTTP。

- [S06 产品需求](logos/resources/prd/1-product-requirements/core-S06-mcp-service-requirements.md)
- [S06 功能设计](logos/resources/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md)
- [MCP 工具契约](logos/resources/api/mcp-tools.yaml)
- [S06 测试用例](logos/resources/test/core-S06-test-cases.md)
- [MCP 构建与四客户端配置](mcp-server/README.md)

### 最近归档变更（2026-06）

| 提案 slug | 类型 | 关键产出 |
|---|---|---|
| `add-frontend-completeness` | B1–B5 五批次 | styles + top menu/toolbar shell + 7-Tab 侧栏 + 5 个核心模态 + 撤销/重做快捷键 |
| `fix-modal-overlay-blocking` | 修复 | ModalRoot 遮罩 + canvas testid + CORS + e2e 修正 |
| `fix-add-frontend-stub-leftover` | 修复 | save handler stubs + selection id wiring + e2e 5/5 |
| `add-local-run-scripts` | 工具 | `scripts/start-local.sh` + `stop-local.sh` 一键启动 |
| `remove-debug-smoke-artifact` | 清理 | 移除 debug 残留 smoke 产物 |
| `wire-editor-canvas` | 重构 | 接线画布到 editor core |
| `redesign-phase-a-layout` | 重构 | V2 布局（AppBar + ToolRail + Inspector + ModalRoot）+ 6 层 z-index |
| `redesign-phase-b-relationship` | 重构 | 关系工具栏 + Tooltip/Popover |
| `redesign-phase-c-import-export` | 重构 | IO 抽屉（替代 V1 Import 模态） |
| `redesign-phase-d-command-code` | 重构 | Command Palette + Code View 规格（已被 E4 Monaco 升级版覆盖） |
| `redesign-phase-e-design-system-migration` | 重构 | E1–E6 设计系统迁移（tokens / icons / components / Monaco / dark mode / motion） |

> 完整归档索引见 `logos/changes/archive/`（15 个已归档提案）。

### 下一步建议

```bash
openlogos next     # 查看下一步推荐
openlogos status   # 查看完整项目状态
```
