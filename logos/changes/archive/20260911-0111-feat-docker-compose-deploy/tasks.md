# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §4/§5 转实现落地口径、§4.1 trunk 路径修正、§6 smoke 入口补实现状态、§9 检查清单对齐

## [code] 代码实现
- [x] `backend`：实现 `GET /api/v1/diagrams/health`（SMOKE-core-01 口径：200 + `{"status":"ok"}`）
- [x] `backend`：静态文件服务 + SPA 回源（`COLDRAWDB_STATIC_DIR` 默认 `/var/www`；非 API 前端路由回源 index.html；wasm/js immutable、index.html no-cache 缓存头）
- [x] `backend`：`COLDRAWDB_BIND_ADDR` / `COLDRAWDB_DB_URL` / `COLDRAWDB_LOG_LEVEL` / `COLDRAWDB_STATIC_DIR` 环境变量覆盖 config.toml（容器内默认 0.0.0.0）
- [x] `backend`：health / 静态服务 / SPA 回源 / 环境变量覆盖的单测（UT-DP-01~05，全绿）
- [x] 根 `Dockerfile` 替换为 Rust 多阶段构建（trunk wasm-build + cargo api-build + debian-slim runtime）
- [x] 新增 `docker-compose.yml`（coldrawdb + nginx + backup）与 `nginx.conf`
- [x] 删除遗留 `compose.yml`、`vercel.json`
- [x] CI `docker.yml`：新增 PR/push 阶段 docker build（不推送）验证步骤
- [x] 部署/静态服务相关测试代码 + OpenLogos reporter（写入 `logos/resources/verify/test-results.jsonl`）

## [deploy] 部署任务
- [x] 本地执行 `docker build -t coldrawdb:v1 .` 验证镜像构建成功
- [x] `docker compose up -d` 启动 staging 形态，确认 healthcheck 通过、`GET /` 返回 index.html
- [x] 确认回滚预案可用（`docker compose down` + 数据卷保留：diagrams 4→4 实测）
- [x] 部署完成后提示用户授权运行 `openlogos smoke`（SMOKE-core-01~05）（已获全权授权；6/6 PASS，Gate 3.8 PASS）
