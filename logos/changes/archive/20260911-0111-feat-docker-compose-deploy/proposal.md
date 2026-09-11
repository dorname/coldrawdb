# 变更提案：补齐 Docker / docker-compose 部署能力

> module: core | created: 2026-09-10

## 变更原因

部署能力调研发现：**本地 dev 链路（start-local.sh）成熟，但容器化部署的规格与实现脱节**——

1. 根 `Dockerfile` 是旧 JS 栈遗留（`npm ci && npm run build` + nginx），仓库根已无 `package.json`，构建必然失败；CI `docker.yml`（tag → ghcr.io 多架构推送）构建的正是这个失效 Dockerfile
2. 后端未实现部署方案承诺的单容器能力：`GET /` 仍返回 `"Hello, world!"`（无静态文件服务、无 SPA 回源），`GET /api/v1/diagrams/health` 不存在（`backend/src/main.rs`）——SMOKE-core-01/04 定义的目标行为当前无法通过
3. 根 `compose.yml` 为旧 dev 栈（挂载 `backend/target/debug/`、端口 6666 与方案 3000 不一致）；`vercel.json` 为不再使用的旧托管配置
4. 后端配置仅支持 `config.toml` 文件（host 固定 127.0.0.1），无方案 §4.3 承诺的环境变量覆盖（容器内必须绑定 0.0.0.0、可配静态目录/DB 路径）

## 变更类型

部署级变更（含代码实现：后端静态服务/health/环境变量覆盖、Dockerfile、compose、CI 修正）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（不改动业务功能）
- 影响的部署方案：`core-01-deployment-plan.md`（§4/§5 从「设计」转为「实现落地」口径；修正 §4.1 与 trunk 实际路径 `frontend-rs/index.html` 的偏差；§6 smoke 入口补充实现状态）
- 影响的 smoke 测试：无内容变更（SMOKE-core-01~05 既定口径即目标行为，本变更让实现达到该口径）
- 影响的业务场景：无（纯部署支撑）
- 影响的 API：新增 `GET /api/v1/diagrams/health`（健康检查，不进 OpenAPI 业务契约，属部署支撑端点，规格已在 smoke 用例中定义）
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：是
- 部署原因：本变更的目的即部署能力本身——需实际执行 `docker build` + `docker compose up` 完成 staging 形态验证，且镜像构建链路（CI docker.yml）随之修复
- 影响环境：本地 / 预发（staging 单机；生产仍为 V1 边界外）
- 是否涉及数据迁移：否（SQLite 文件卷挂载，schema 由既有 migration 自动执行）
- 是否需要回滚预案：是（`docker compose down` + 删除新文件即回滚；不影响本地 dev 链路）
- 是否需要 smoke：是（部署后运行 `openlogos smoke`，执行 SMOKE-core-01~05）

## 变更概述

1. **后端补齐单容器能力**：实现 `GET /api/v1/diagrams/health`（200 `{"status":"ok"}`）；静态文件服务（`COLDRAWDB_STATIC_DIR`，默认 `/var/www`）+ 非 API 前端路由回源 `index.html`（§12.3 SPA 回源）；WASM/JS 缓存头 immutable、`index.html` no-cache（§3.4.6）；`COLDRAWDB_BIND_ADDR` / `COLDRAWDB_DB_URL` / `COLDRAWDB_LOG_LEVEL` / `COLDRAWDB_STATIC_DIR` 环境变量覆盖 config.toml（§4.3），容器内默认绑定 0.0.0.0。
2. **容器产物**：根 `Dockerfile` 替换为方案 §4.1 的 Rust 多阶段构建（trunk wasm-build → cargo api-build → debian-slim runtime，修正 trunk 实际路径 `frontend-rs/index.html`）；新增 staging 用 `docker-compose.yml`（coldrawdb + nginx + backup，§5.1）与 `nginx.conf`（§5.2 + 缓存头规则）；删除遗留 `compose.yml` 与 `vercel.json`。
3. **CI 修正**：`docker.yml` 随根 Dockerfile 替换自动生效；新增 PR/push 阶段的 docker build（不推送）验证步骤，防止镜像构建再次静默腐坏。
