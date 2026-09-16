# Delta — core-smoke-test-cases.md（稳定版 Compose 冒烟）

> 变更：release-stable-win-linux-mac / v0.1.0

## ADDED — SMOKE-core-STABLE-01 — 稳定版 Compose 健康检查

### 目的

验证用户按稳定版入口（Docker Compose）启动后，公开入口与后端健康检查可用。

### 前置

- 本机 Docker daemon 可用
- 仓库根目录或已发布源码包

### 步骤

1. `docker compose up -d --build`（或 `docker pull` + 等价 run）
2. 等待 `coldrawdb` healthcheck 健康
3. `GET http://localhost/api/v1/diagrams/health`（经 nginx:80）→ 200 + `status=ok`
4. `GET http://localhost/` → SPA 入口可达（非 502）

### 断言

- compose 服务 `coldrawdb` / `nginx` running
- health 200
- 首页 HTTP < 500

### 环境不可用时

若执行机无 Docker daemon：在 `deployment-report.md` / smoke 报告标注 `SKIPPED: no docker daemon`，并以 GitHub Actions `docker.yml` tag job（双架构 push）成功作为替代发布证据；**不得**将本 skip 记为产品缺陷。

## MODIFIED — 用例 ID 清单

| ID | 标题 |
|---|---|
| SMOKE-core-STABLE-01 | 稳定版 Compose 健康检查 |
