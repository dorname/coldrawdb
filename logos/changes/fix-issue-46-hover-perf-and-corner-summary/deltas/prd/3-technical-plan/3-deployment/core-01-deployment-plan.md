# Delta — core-01-deployment-plan.md（fix-issue-46-hover-perf-and-corner-summary / #46）

## ADDED — fix-issue-46 部署注记

- 本变更为纯前端画布代码（hover 信号瘦身 + 守卫 + rAF 节流 + tooltip 角落定位），**无 DB migration、无 API 变更**。
- 部署形态：`docker compose up -d --build coldrawdb` 重建镜像（前端产物随镜像），nginx/backup 不变。
- 部署顺序无要求：纯客户端展示/性能逻辑，对旧后端完全兼容。
- 回滚：回退上一镜像即可，无数据迁移。
- 部署后 smoke：既有套件全量回归（不新增用例）；SMOKE-core-STABLE-01 沿用「部署先行」判定。
