# Delta — core-01-deployment-plan.md（fix-font-lod-consistency-and-table-resize）

## ADDED — fix-font-lod-consistency-and-table-resize 部署注记

- 本变更为纯前端画布渲染逻辑（表卡尺寸随 effective 字号自适应 + 精灵缓存一致性），**无 DB migration、无 API 变更**。
- 部署形态：`docker compose up -d --build coldrawdb` 重建镜像（前端产物随镜像），nginx/backup 不变。
- 部署顺序无要求：纯客户端渲染逻辑，对旧后端完全兼容。
- 回滚：回退上一镜像即可，无数据迁移。
- 部署后 smoke：既有套件全量回归（不新增用例）；SMOKE-core-STABLE-01 沿用「部署先行」判定。
