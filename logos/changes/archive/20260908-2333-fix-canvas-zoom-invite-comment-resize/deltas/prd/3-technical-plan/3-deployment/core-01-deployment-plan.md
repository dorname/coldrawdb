# Delta: core-01-deployment-plan.md

> 提案：fix-canvas-zoom-invite-comment-resize（问题 2：协作邀请链接无效）

## ADDED — ## 12. 公开访问基址（PUBLIC_BASE_URL）与 SPA 路由回源

## 12. 公开访问基址（PUBLIC_BASE_URL）与 SPA 路由回源

> 新增于 fix-canvas-zoom-invite-comment-resize：修复后端 `invite_url` 硬编码 `http://localhost/invite/{token}`（无端口、不可配置）与前端各 Client base_url 硬编码 `http://127.0.0.1:3000` 导致的「邀请链接跨机不可访问」问题。

### 12.1 配置项

| 变量 | 默认 | 说明 |
|---|---|---|
| `PUBLIC_BASE_URL` | 空（未配置） | 后端生成对外完整 URL（当前用于邀请链接 `invite_url`）所用的公开基址，如 `https://coldrawdb.example.com` 或含端口 `http://192.168.1.10:3000` |

语义与推导规则：

1. 已配置 `PUBLIC_BASE_URL` → 后端以其为基址生成 `invite_url = {PUBLIC_BASE_URL}/invite/{token}`。
2. 未配置 → 后端回退以**请求 Host header + scheme 推导**（`Forwarded` / `X-Forwarded-Proto` 优先），保留非默认端口。
3. **禁止**硬编码 `localhost` / `127.0.0.1` / 无端口占位 host——生成的链接必须可被公网 / 局域网内其他机器直接打开。

### 12.2 前端 base_url 派生

前端 `DiagramClient` / `AuthClient` / `RoomClient` / `CollabClient` 的 base_url 一律由 `window.location.origin` 同源派生（单入口部署下前后端同源，无需额外配置）。仅本地 dev 双进程（trunk serve :8080 + cargo run :3000）允许经环境变量 / 构建配置覆盖为 `http://127.0.0.1:3000`；该覆盖不得进入生产构建默认值。

### 12.3 单容器 / 反代场景的 SPA 回源

单容器镜像（backend 同时服务 `/var/www` 静态资源与 `/api/v1/*`）与 staging nginx 反代均为单入口。必须保证下列前端路由回源 SPA（返回 `index.html`），不得 404：

- `/invite/*`（邀请接受页）
- `/editor/*`（编辑器）
- `/rooms*`（房间列表）
- `/login*` 及其余前端路由

nginx 反代示例（`/api/` 透传后端，其余回源后端静态服务；后端对非 API 路径回源 `index.html`）：

```nginx
location /api/ { proxy_pass http://backend; }
location / { proxy_pass http://backend; }
```

部署检查清单追加：

- [ ] `PUBLIC_BASE_URL` 已配置，或 Host 推导在反代下含正确 scheme 与端口（`Forwarded` / `X-Forwarded-Proto` 已透传）
- [ ] 生成邀请链接跨机可打开：`/invite/{token}` 页面 200 且 `GET /api/v1/rooms/invites/{token}` preview 200
