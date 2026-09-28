# Delta — core-01-deployment-plan.md（fix-issues-42-44-mcp-and-relation-hit）

## ADDED — fix-issues-42-44 部署注记

- 本变更为纯前端画布代码（#43 悬浮 tooltip / #44 关系线命中）+ mcp-server 客户端错误处理（#42），**无 DB migration、无新 API 端点**。
- 部署形态：`docker compose up -d --build coldrawdb` 重建镜像（前端产物随镜像），nginx/backup 不变；mcp-server 为 stdio 本机进程，随客户端分发，**不经 compose 部署**。
- 部署顺序无要求：前端对旧后端完全兼容（命中/悬浮均为客户端纯前端逻辑）。
- 回滚：移除新镜像回退上一镜像即可，无数据迁移。
- 部署后 smoke：既有套件全量回归（不新增用例）；SMOKE-core-STABLE-01 沿用「部署先行」判定（栈已在运行时直接验证）。
