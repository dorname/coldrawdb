# Delta — core-smoke-test-cases.md（修复 V2 路由可达性）

> module: core | proposal: fix-backend-v1-scope-routing

## ADDED — 11. SMOKE-core-07 — V2 auth 路由可达性

### 11.1 目的

验证 `/api/v1/auth/register` 与 `/api/v1/auth/login` 不再因重复 scope 挂载而返回 404。

### 11.2 步骤

1. 启动后端（本地 staging 等价环境）。
2. `POST /api/v1/auth/register` body:
   ```json
   {
     "email": "smoke-test@example.com",
     "password": "smoke-password-123",
     "display_name": "smoke"
   }
   ```
   → 期望 201 或 409（邮箱已存在）；绝不能是 404。
3. `POST /api/v1/auth/login` body:
   ```json
   {
     "email": "smoke-test@example.com",
     "password": "smoke-password-123"
   }
   ```
   → 期望 200 或 401（凭据错误）；绝不能是 404。

### 11.3 断言

- 两个端点均不返回 404。
- 响应包含 `Content-Type: application/json`。
- 若注册成功，登录返回 200 并含 `access_token`。

### 11.4 失败处理

- 返回 404 → `backend/src/main.rs` 仍存在重复 `/api/v1` scope，需重新合并。
- 返回 500 → 检查后端日志与数据库连接。
