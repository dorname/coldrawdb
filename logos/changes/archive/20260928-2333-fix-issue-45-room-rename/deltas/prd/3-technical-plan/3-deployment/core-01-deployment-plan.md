# Delta — core-01-deployment-plan.md（fix-issue-45-room-rename / #45）

## ADDED — fix-issue-45 部署注记

- 本变更新增后端端点 `PATCH /api/v1/rooms/{roomId}`（renameRoom）+ 前端房间列表卡片「重命名」入口，**无 DB migration、无破坏性 API 变更**（纯增量端点）。
- 部署形态：`docker compose up -d --build coldrawdb` 重建镜像（前后端产物随镜像），nginx/backup 不变。
- 部署顺序无要求：旧前端不调新端点，新前端对旧后端仅「重命名」入口 404/405（入口为新增，无回归面）。
- 回滚：回退上一镜像即可，无数据迁移。
- 部署后 smoke：既有套件全量回归（不新增用例）；SMOKE-core-STABLE-01 沿用「部署先行」判定（栈已在运行时直接验证）。
