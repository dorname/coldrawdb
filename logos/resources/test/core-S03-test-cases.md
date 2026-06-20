## 1. 范围

本文件覆盖场景 S03（用户注册 / 登录 / Token 续期）的 UT 与 ST 用例规格。

**对应实现**：`backend/src/auth_v1.rs` + `backend/src/auth/*`

**API 契约**：`logos/resources/api/auth.yaml`

**DDL**：`logos/resources/database/coldrawdb-v2-auth.sql`

## 2. UT 用例

### UT-S03-01 — 注册成功 201

- **位置**：`auth_v1::tests::ut_s03_01_register_success`
- **步骤**：POST `/api/v1/auth/register` 合法 email/password
- **断言**：201；`userId` UUID；`email` 匹配

### UT-S03-02 — 重复邮箱 409

- **位置**：`auth_v1::tests::ut_s03_02_register_duplicate_email`
- **断言**：409；`code == EMAIL_EXISTS`

### UT-S03-03 — 登录成功 Set-Cookie

- **位置**：`auth_v1::tests::ut_s03_03_login_success_sets_cookie`
- **断言**：200；`accessToken` 存在；`Set-Cookie` 含 refresh_token

### UT-S03-04 — 错误密码 401

- **位置**：`auth_v1::tests::ut_s03_04_login_invalid_password`
- **断言**：401；`code == INVALID_CREDENTIALS`

### UT-S03-05 — refresh 成功

- **位置**：`auth_v1::tests::ut_s03_05_refresh_success`
- **断言**：200；新 accessToken

### UT-S03-06 — refresh 无效 401

- **位置**：`auth_v1::tests::ut_s03_06_refresh_invalid`
- **断言**：401；`code == REFRESH_INVALID`

### UT-S03-07 — logout 撤销 refresh

- **位置**：`auth_v1::tests::ut_s03_07_logout_revokes_refresh`
- **断言**：logout 204；后续 refresh 401

## 3. ST 用例

### ST-S03-01 — 注册→登录→/me

- **位置**：`auth_v1::tests::st_s03_01_register_login_me_flow`
- **步骤**：register → login → GET `/api/v1/auth/me` Bearer
- **断言**：profile email/displayName 正确
