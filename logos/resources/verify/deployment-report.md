# 部署报告

## 2026-09-21 — fix-diagram-import-persistence（本地/测试）

| 项 | 值 |
|---|---|
| 变更 | `fix-diagram-import-persistence` |
| 部署时间 | 2026-09-21 |
| 目标环境 | 本地 docker compose（coldrawdb + nginx + backup） |
| 版本 | `v0.2.0`（镜像 `coldrawdb:v1` = `coldrawdb:0.2.0`，216237b25228，修复后构建） |
| 状态 | ✅ 部署完成；smoke Gate 3.8 PASS（8/8，Coverage 100%） |

### 执行摘要

1. 后端修复（导入持久化逐表容错 + 缺 id 自动生成 + name 缺省兜底 + 持久化错误传播）随镜像重建部署；容器实测：缺 id payload 导入 → `imported_tables=1`、auto- id 补全、`name="imported_diagram"`、`revision=1`、warnings 明细
2. 存量数据迁移：`UPDATE diagram SET name='imported_diagram' WHERE name IS NULL` → affected_rows=0（本库无脏数据，空转幂等）
3. `openlogos verify` Gate 3.6 PASS（458/458）
4. `openlogos smoke` Gate 3.8 PASS（8/8）：含本提案新增 SMOKE-core-07（导入缺 id payload 持久化验证，112ms）与 SMOKE-core-STABLE-01（compose 经 nginx :9080 健康检查，15.4s）
5. `docker compose up -d`：`coldrawdb` Healthy、`nginx` Started；`GET :9080/api/v1/diagrams/health` → 200 `{"status":"ok"}`；`GET :9080/` SPA 入口 → 200
6. smoke runner 扩展：`scripts/smoke-local-scripts.sh` 新增 Stage 6b（SMOKE-core-07）与 Stage 8（SMOKE-core-STABLE-01），覆盖率 75% → 100%

### 回滚点

- `docker compose down` 停止 compose 栈；开发态可用 `scripts/start-local.sh` 恢复宿主机服务
- 迁移为幂等 UPDATE，无回滚需求；旧版本后端可继续运行（nullable 契约为防御性兼容）

### 备注 / 未解决风险

- smoke 沙箱写保护会拦截 runner 在沙箱副本内的冷编译（cargo/trunk 写 target 被拒/副本无 target 全量冷编超限）。本次经用户授权，以 `OPENLOGOS_SANDBOX_WRITE_PROTECTION=off` + `COLDRAWDB_FRONTEND_TIMEOUT=300` 一次性环境变量执行，未修改任何配置文件；沙箱状态记 `warn`，不影响 Gate 判定
- 对外入口：http://localhost:9080/（nginx）；后端直连 :3000

---

# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `release-deploy-bundle` |
| 部署时间 | 2026-09-18 |
| 目标环境 | GitHub Release（用户本机 Docker：Win/macOS/Linux）+ GHCR |
| 版本 | `v0.1.0`（覆盖重发） |
| 状态 | Release 已就绪（deploy.zip 资产就位）；GHCR 多架构推送（Docker #16）后台进行中 |

## 执行摘要

1. 规格已合并：稳定版交付物由「源码 zip」改为 **`coldrawdb-<tag>-deploy.zip`**（compose.yml 引用 GHCR 预构建镜像 + nginx.conf + .env.sample + 快速上手.md）
2. `openlogos verify` Gate 3.6 PASS（431/431，Coverage 100%）
3. 已推送 `main`（`9cc9f29`），force 移动 annotated tag `v0.1.0`（`c2ad9b7...89a9f2f`）
4. Release #4 自动重建（10s 完成），body 指向 deploy.zip 并注明自动归档非交付物
5. 旧 `coldrawdb-v0.1.0-src.zip` 资产已从 Release 移除（删除时 PAT 权限不足，但 softprops 覆盖 Release 时已清理；API 资产列表与页面均已确认只剩 deploy.zip）
6. Docker Build and Push #16（tag v0.1.0，多架构推 GHCR）后台进行中（上次同型构建耗时约 3h，预计 2026-09-18 上午完成）
7. 本机无 Docker daemon：SMOKE-core-STABLE-01 记 `SKIPPED: no docker daemon`（已追加至 smoke-results.jsonl），以 CI 证据代替

## 回滚点

- tag / Release 可 force 回上一 SHA（`c2ad9b7`，compose-default-port-9080 完成态）
- 或临时在 Release 页面手动上传旧源码 zip

## 访问

- Release：https://github.com/dorname/coldrawdb/releases/tag/v0.1.0
- 镜像（CI 完成后）：`ghcr.io/dorname/coldrawdb:v0.1.0`（linux/amd64 + linux/arm64）
- 使用：下载 deploy.zip → 解压 → `docker compose up -d` → http://localhost:9080/
