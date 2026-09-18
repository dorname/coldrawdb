# Delta — core-smoke-test-cases.md（SMOKE-core-STABLE-01 端口 9080）

> 变更：compose-default-port-9080

## MODIFIED — §8.5 SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

### 目的

验证用户按稳定版入口（Docker Compose）启动后，公开入口与后端健康检查可用。

### 前置

- 本机 Docker daemon 可用
- 仓库根目录或已发布源码包
- 使用默认 compose 映射（`COLDRAWDB_HTTP_PORT` 未覆盖时为 **9080**）

### 步骤

1. `docker compose up -d --build`（或 `docker pull` + 等价 run）
2. 等待 `coldrawdb` healthcheck 健康
3. `GET http://localhost:9080/api/v1/diagrams/health`（经 nginx，宿主机 9080）→ 200 + `status=ok`
4. `GET http://localhost:9080/` → SPA 入口可达（非 502）

### 断言

- compose 服务 `coldrawdb` / `nginx` running
- health 200
- 首页 HTTP < 500

### 环境不可用时

若执行机无 Docker daemon：在 `deployment-report.md` / smoke 报告标注 `SKIPPED: no docker daemon`，并以 GitHub Actions `docker.yml` tag job（双架构 push）成功作为替代发布证据；**不得**将本 skip 记为产品缺陷。
