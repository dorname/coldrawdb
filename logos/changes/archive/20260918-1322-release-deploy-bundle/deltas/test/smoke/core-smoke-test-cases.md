# Delta: core-smoke-test-cases.md — release-deploy-bundle（SMOKE-core-STABLE-01 以 deploy.zip 为起点）

## MODIFIED — 8.5 SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

## 8.5 SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

### 目的

验证用户按稳定版入口（Release 资产 `coldrawdb-<tag>-deploy.zip` + Compose）启动后，公开入口与后端健康检查可用。

### 前置

- 本机 Docker daemon 可用
- Release 资产 `coldrawdb-<tag>-deploy.zip`（或其解压目录）
- 使用默认 compose 映射（`COLDRAWDB_HTTP_PORT` 未覆盖时为 **9080**）

### 步骤

1. 解压 `coldrawdb-<tag>-deploy.zip` 到独立目录，进入该目录
2. `docker compose up -d`（release 形态 compose 引用 GHCR 预构建镜像，**无** `--build`）
3. 等待 `coldrawdb` healthcheck 健康
4. `GET http://localhost:9080/api/v1/diagrams/health`（经 nginx，宿主机 9080）→ 200 + `status=ok`
5. `GET http://localhost:9080/` → SPA 入口可达（非 502）

### 断言

- compose 服务 `coldrawdb` / `nginx` running
- health 200
- 首页 HTTP < 500

### 环境不可用时

若执行机无 Docker daemon：在 `deployment-report.md` / smoke 报告标注 `SKIPPED: no docker daemon`，并以 GitHub Actions `docker.yml` tag job（双架构 push）+ `release.yml` job（deploy.zip 资产就绪）成功作为替代发布证据；**不得**将本 skip 记为产品缺陷。
