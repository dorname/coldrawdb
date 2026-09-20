# Delta — core-S03-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#16（access token TTL 配置化，默认 3600s） | 规格：`api/auth.yaml` TTL 不变量

## ADDED — UT-S03-08 — access TTL 配置化与默认值

- **位置**：`backend/src/auth/jwt.rs` + `backend/src/init.rs`（`[auth] access_ttl_secs`）
- **断言**：
  - 配置缺省 → 签发 JWT 的 `exp - iat == 3600`
  - `config.toml` 设 `access_ttl_secs = 1800` → `exp - iat == 1800`
  - env `COLDRAWDB_ACCESS_TTL_SECS=7200` 覆盖 TOML → `exp - iat == 7200`
  - 非法值（0 / 负数 / 超 86400 上限）→ 启动报错，不静默回退

## ADDED — UT-S03-09 — expiresIn 与 JWT exp 一致

- **位置**：`backend/src/auth_v1.rs`（login / refresh handler）
- **断言**：POST `/api/v1/auth/login` 与 `/auth/refresh` 返回体的 `expiresIn` == 生效配置值 == 响应 JWT 的 `exp - iat`（解码头(payload) 校验）

## ADDED — ST-S03-02 — 续期链路在默认 TTL 下稳定

- **步骤**：默认配置启动 → 登录 → 携带过期 accessToken + 有效 refresh Cookie 调 `/auth/refresh` → 用新 accessToken 访问 `/auth/me`
- **断言**：refresh 200 且 `expiresIn == 3600`；`/auth/me` 200；旧 refresh_token 已撤销（复用 401）

## ADDED — TTL 口径说明（随 UT-S03-08/09 生效）

- 既有用例中含「15m / 900」字样的 TTL 预期统一改为「默认 3600，可配置」；不依赖具体秒数的用例不受影响。

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S03"`）。
