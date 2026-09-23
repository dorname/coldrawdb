# Delta：core-smoke-test-cases.md（compose-backend-port-internal-only）

## MODIFIED — 8.5 SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

## 8.5 SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

### 目的

验证用户按稳定版入口（Release 资产 `coldrawdb-<tag>-deploy.zip` + Compose）启动后，公开入口与后端健康检查可用；并验证后端端口未暴露宿主机（compose-backend-port-internal-only：对外唯一入口为 nginx 9080，消除 3000/9080 双入口迷惑）。

### 前置

- 本机 Docker daemon 可用
- Release 资产 `coldrawdb-<tag>-deploy.zip`（或其解压目录；本仓库开发形态 `docker-compose.yml` 等同验证）
- 使用默认 compose 映射（`COLDRAWDB_HTTP_PORT` 未覆盖时为 **9080**）

### 步骤

1. 解压 `coldrawdb-<tag>-deploy.zip` 到独立目录，进入该目录
2. `docker compose up -d`（release 形态 compose 引用 GHCR 预构建镜像，**无** `--build`）
3. 等待 `coldrawdb` healthcheck 健康
4. `GET http://localhost:9080/api/v1/diagrams/health`（经 nginx，宿主机 9080）→ 200 + `status=ok`
5. `GET http://localhost:9080/` → SPA 入口可达（非 502）
6. `curl http://localhost:3000/api/v1/diagrams/health`（或任一 TCP 连接尝试）→ **应连接失败**：后端未映射宿主机端口，宿主 3000 无监听

### 断言

- compose 服务 `coldrawdb` / `nginx` running
- health 200
- 首页 HTTP < 500
- 宿主机 3000 无监听（连接失败 / refused）——防止后端宿主端口映射回潮

### 环境不可用时

若执行机无 Docker daemon：在 `deployment-report.md` / smoke 报告标注 `SKIPPED: no docker daemon`，并以 GitHub Actions `docker.yml` tag job（双架构 push）+ `release.yml` job（deploy.zip 资产就绪）成功作为替代发布证据；**不得**将本 skip 记为产品缺陷。
