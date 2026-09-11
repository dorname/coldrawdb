# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `fix-diagram-entity-persistence` |
| 部署时间 | 2026-06-21 |
| 目标环境 | 本地开发 |
| 状态 | 成功 |

## 执行摘要

1. `./scripts/start-local.sh` — 启动后端（`:3000`）与前端 trunk serve（`:8080`）
2. 健康检查：`http://127.0.0.1:3000/` 与 `http://127.0.0.1:8080/` 均就绪
3. `openlogos smoke fix-diagram-entity-persistence` — 6/6 通过（Gate 3.8 PASS）

## 迁移

无数据迁移（SQLite schema 未变更）。

## 回滚点

- 停止服务：`./scripts/stop-local.sh`
- 代码回滚：revert 本变更相关 commit

## 访问地址

- 前端：http://127.0.0.1:8080/editor
- 后端：http://127.0.0.1:3000/

## 风险

无未解决风险。

---

# feat-docker-compose-deploy 部署记录（2026-09-10）

## 环境

staging 单机（本机 Docker）：`docker compose up -d`（coldrawdb + nginx + backup）

## 执行步骤与证据

1. `docker build -t coldrawdb:v1 .` — 成功（Rust 1.89 多阶段：trunk wasm-build + cargo api-build + debian-slim runtime；修复：依赖树含 edition2024 包需 ≥1.85；apt 构建期代理 502 → 构建期 unset proxy + Acquire::Retries=5；trunk 0.21.14 无 --no-wasm-opt flag，走 index.html data-wasm-opt="0"）
2. `docker compose up -d` — coldrawdb healthy（healthcheck `GET /api/v1/diagrams/health` 通过），nginx :80、backup 正常启动
3. 容器内 smoke 实测（manual，hit 容器 :3000 与 nginx :80）：
   - SMOKE-core-01：`GET :3000/api/v1/diagrams/health` → 200 `{"status":"ok"}`，6ms（<500ms）
   - SMOKE-core-02：create→read→update(revision 0→1)→delete→404 全链路通过
   - SMOKE-core-03：`/bridge/import/local` 结构化 payload → diagram_id/log_id + status=success；GET diagram 验证 tables[0]=smoke_users、2 字段（首次失败原因为复用 SMOKE-02 的表 ID 主键冲突，改用全局唯一 ID 后通过；smoke 文档契约已对齐）
   - SMOKE-core-04：`GET /` 与 `/index.html` → 200 含 `<div id="app">` + `<script type="module">`，no-cache；`/editor` SPA 回源 200；`frontend-rs-*.js` 200 text/javascript immutable；`*_bg.wasm` 200 application/wasm 1.8MB(>100KB) immutable
   - SMOKE-core-05：SQLite 含全部 11 张核心表（共 21 张，migration 0001~0008 全应用）；直写/直删 diagram 行成功；磁盘余量 >400GB
   - nginx :80 路径：`/`、`/editor`、`/api/v1/diagrams/health` 均 200
4. 回滚演练：`docker compose down` → `up -d`，数据卷 `./data` 保留（diagrams 4→4），容器 healthy
5. `openlogos smoke`（本地脚本链路，SMOKE-core-01~06）— 6/6 通过（Gate 3.8 PASS）

## 迁移

无数据迁移（SQLite 文件卷挂载，schema 由 migration 0001~0008 自动执行）。

## 回滚点

- `docker compose down`（数据卷 ./data 保留）
- 代码回滚：revert 本变更相关 commit

## 访问地址

- 容器直连：http://127.0.0.1:3000/（API + SPA 单入口）
- nginx 反代：http://127.0.0.1:80/

## 风险

- smoke 脚本链路（smoke-local-scripts.sh）与 docker staging 同占 :3000，二者需分时运行（本次 smoke 前先 compose down、后恢复）
- `backup` 服务每日打包 ./data，长期运行需关注 ./backups 磁盘占用
