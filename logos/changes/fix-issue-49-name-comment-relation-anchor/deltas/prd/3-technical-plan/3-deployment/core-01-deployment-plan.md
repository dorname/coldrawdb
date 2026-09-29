# Delta — core-01-deployment-plan.md（fix-issue-49-name-comment-relation-anchor）

## ADDED — fix-issue-49-name-comment-relation-anchor 部署注记

- 本变更为纯前端画布代码（关系线锚点/选侧统一使用当前 `comment_mode`，与表体绘制口径一致），**无 DB migration、无 API 变更**。
- 部署形态：`docker compose up -d --build coldrawdb` 重建镜像（前端产物随镜像），nginx/backup 不变。
- 部署顺序无要求：纯客户端渲染/锚点逻辑，对旧后端完全兼容。
- 回滚：回退上一镜像即可，无数据迁移。
- 部署后 smoke：既有套件全量回归（不新增用例）；SMOKE-core-STABLE-01 沿用「部署先行」判定。
