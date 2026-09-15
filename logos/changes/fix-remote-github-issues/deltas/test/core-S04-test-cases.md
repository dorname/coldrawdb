# Delta — core-S04-test-cases.md（#1 invite 基址 = 公开入口）

> 变更：fix-remote-github-issues / GitHub #1

## ADDED — UT-S04-18 — compose 默认 PUBLIC_BASE_URL 指向公开入口（非 :3000）

### UT-S04-18 — staging 默认基址语义

- **位置**：部署脚本 / `docker-compose.yml` 静态断言，或后端在 `PUBLIC_BASE_URL=http://localhost` 下创建 invite
- **步骤**：
  1. 读取 compose 文件：`coldrawdb.environment` 含 `PUBLIC_BASE_URL`
  2. 默认值**不得**为 `*:3000` 裸后端调试口
  3. 设置 `PUBLIC_BASE_URL=http://192.168.20.106` → `POST /rooms/{id}/invites`
- **断言**：
  - `inviteUrl` 前缀为 `http://192.168.20.106/invite/`
  - 不含 `:3000`（除非操作者显式把公开入口配成带 3000 的单入口）
  - reporter 登记 `UT-S04-18`

## MODIFIED — UT-S04-16 说明脚注

UT-S04-16 使用 `PUBLIC_BASE_URL=http://192.168.1.10:3000` 仅验证「env 优先于 Host」的**机制**；**不**表示生产推荐把邀请基址设为后端端口。staging/prod 推荐值见部署方案 §12.5（公开 SPA 入口）。

## MODIFIED — 用例 ID 清单（附录）

| ID | 标题 |
|---|---|
| UT-S04-18 | compose/公开入口 PUBLIC_BASE_URL；inviteUrl 非默认裸 :3000 |
