# S03：用户注册 / 登录 / Token 续期 — 交互设计

> 模块：core | 场景：S03 | 版本：V2 | 优先级：P2
> 原型：`core-03-auth-prototype.html`
> Phase 1 输入：`core-00-scenario-overview.md` §S03 / `core-01-requirements.md` US 关联 P03
> 参考：drawdb main **无用户鉴权**（V1 完全匿名）；本场景为 coldrawdb V2 net-new，为 S04/S05 前置

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（SaaS 鉴权层 + 编辑器入口） |
| 原型形式 | 可交互 HTML（登录 / 注册 / 会话续期三视图） |
| 视觉基准 | 延续 `core-07` token（主色 `#175e7a`）+ Plus Jakarta Sans |
| 与 main 关系 | drawdb 无对应页面；交互模式参考常见 B2B SaaS（单栏表单 + 品牌侧栏） |

## 2. 信息架构（V2 增量）

### 2.1 路由

| 路由 | 页面 | 未登录 | 已登录 |
|---|---|---|---|
| `/login` | 登录 | ✅ 可访问 | 重定向 `/editor` |
| `/register` | 注册 | ✅ 可访问 | 重定向 `/editor` |
| `/verify-email` | 邮箱验证等待 / 确认 | ✅（带 `?token=`） | 可访问 |
| `/editor` | 编辑器 | V2：**需登录**（私有 diagram） | ✅ |
| `/editor?share=` | 分享只读 | ✅ 匿名只读（延续 S02） | ✅ |

### 2.2 鉴权状态机

```
[匿名] ──注册成功──→ [待验证邮箱?] ──验证──→ [已登录]
   │                      │
   └──登录成功─────────────┘
                              │
                    access_token 过期
                              │
                    refresh 成功 ──→ [已登录]
                    refresh 失败 ──→ [匿名] + 跳转 /login?redirect=...
```

### 2.3 数据实体（Phase 3 预埋）

| 实体 | 字段（交互相关） | 说明 |
|---|---|---|
| `user` | `id`, `email`, `password_hash`, `display_name`, `email_verified_at` | 密码 Argon2id 哈希，前端永不接触明文哈希 |
| `auth_token` | `id`, `user_id`, `refresh_token_hash`, `expires_at`, `revoked_at` | 支持多设备 refresh；logout 写 `revoked_at` |

## 3. 子场景与交互流程

### S03.1 注册

**涉及页面**：`/register`

**表单字段**：

| 字段 | 类型 | 必填 | 校验 |
|---|---|---|---|
| email | email | ✅ | RFC 格式；服务端查重 |
| display_name | text | ❌ | 长度 ≤ 32 |
| password | password | ✅ | ≥ 8 位，含字母 + 数字 |
| confirm_password | password | ✅ | 与 password 一致 |

**交互流程**：

1. 用户从 Landing / 编辑器拦截页点击「注册」→ `/register`
2. 实时校验：邮箱格式、密码强度条、确认密码一致
3. 提交 → 按钮 loading → `POST /api/v1/auth/register`
4. **201** → 若启用邮箱验证：跳转 `/verify-email?sent=1`；否则自动登录跳转 `/editor`
5. **409**（邮箱已注册）→ 字段下方红字 + 「去登录」链接
6. **422** → 字段级错误映射

**原型锚点**：`[data-testid="register-form"]` / `[data-testid="register-submit"]`

---

### S03.2 登录

**涉及页面**：`/login`

**表单字段**：email + password；可选「记住此设备」（延长 refresh 有效期）

**交互流程**：

1. 用户访问 `/login`（或 401 拦截带 `?redirect=/editor/xxx`）
2. 填写凭据 → `POST /api/v1/auth/login`
3. **200** → 响应 `access_token`（JSON）+ `Set-Cookie: refresh_token`（HttpOnly, Secure, SameSite=Lax）
4. 前端存 `access_token` 至 memory（或 sessionStorage，**禁止 localStorage**）
5. 跳转 `redirect` 参数或默认 `/editor`
6. **401** → 「邮箱或密码错误」（不区分 email 是否存在，防枚举）
7. **429** → 「尝试过多，请 N 分钟后重试」+ 禁用提交按钮

**登录后 AppBar 变化**（编辑器内）：

- 新增 `[data-testid="user-menu"]`：显示 display_name 首字母头像
- 下拉：个人设置 / 退出登录
- 「分享」私有 diagram 时可选择「仅登录用户可编辑」

**原型锚点**：`[data-testid="login-form"]` / `[data-testid="login-submit"]` / `[data-testid="user-menu"]`

---

### S03.3 Token 续期（Refresh）

**触发条件**：

- `access_token` 剩余 TTL < 60s 时 proactive refresh
- 任意 API 返回 **401** 且 body `{ "code": "token_expired" }` 时 reactive refresh

**交互流程**（对用户无感，除非失败）：

1. 前端拦截器捕获 401 + `token_expired`
2. 若尚无进行中的 refresh：``POST /api/v1/auth/refresh``（cookie 自动携带 refresh_token）
3. **200** → 更新 memory 中 `access_token` → **重放**失败的原请求（用户无感知）
4. **401**（refresh 无效 / 已撤销）→ 清除会话 → Toast「登录已过期，请重新登录」→ `/login?redirect=<current>`
5. 并发请求：仅一次 refresh，其余排队等待（mutex）

**可见 UI 反馈**（仅调试 / 原型演示）：

- StatusBar 或 user-menu 旁 `[data-testid="session-indicator"]`：`会话有效 · 续期 14:32`
- refresh 进行中：指示点 amber 闪烁（≤ 300ms，不阻塞编辑）

**原型锚点**：`[data-testid="session-indicator"]` / 按钮「模拟 Token 过期」触发续期动画

---

### S03.4 退出登录

1. user-menu →「退出登录」
2. `POST /api/v1/auth/logout` → 服务端 revoke refresh_token
3. 前端清除 access_token → 跳转 `/login`
4. 若用户在私有 editor 页：保存 debounce 完成后再退出（有未保存则 Confirm 模态）

## 4. 与 V1 / S01 / S02 的衔接

| V1 行为 | V2 + S03 后 |
|---|---|
| diagram 按 id 无鉴权读写 | 私有 diagram 需 `Authorization: Bearer`；owner 校验 |
| `?share=` 匿名只读 | **保持不变**（S02 兼容） |
| AppBar 无用户区 | 增加 user-menu + session 指示 |
| PUT 409 冲突 | 冲突双方需为同一 owner 或 room 成员（S04 扩展） |

## 5. 验收条件（交互级）

##### 正常：注册并进入编辑器

- **GIVEN** 用户在 `[data-testid="register-form"]`，邮箱未被占用
- **WHEN** 用户填写 `dev@example.com`、密码 `Pass1234`、确认密码一致，点击 `[data-testid="register-submit"]`
- **THEN**
  - 提交按钮 loading 1–2s
  - 成功跳转编辑器或验证邮件页
  - AppBar 出现 `[data-testid="user-menu"]`

##### 正常：登录带 redirect

- **GIVEN** 用户未登录，访问 `/login?redirect=/editor/d-abc`
- **WHEN** 用户输入正确凭据并提交 `[data-testid="login-submit"]`
- **THEN** 跳转 `/editor/d-abc`，且后续 diagram API 请求含 `Authorization` 头

##### 正常：Token 静默续期

- **GIVEN** 用户已登录，`access_token` 即将过期
- **WHEN** 用户继续编辑触发 PUT（S01 debounce）
- **THEN**
  - 前端自动 refresh，PUT 成功，用户无弹窗
  - `[data-testid="session-indicator"]` 更新续期时间
  - 编辑不中断

##### 异常：refresh 失败强制重新登录

- **GIVEN** refresh_token 已被 revoke（另一设备退出）
- **WHEN** 用户触发任意 API
- **THEN**
  - Toast「登录已过期，请重新登录」
  - 跳转 `/login?redirect=<current_path>`
  - 本地 access_token 已清除

##### 异常：登录失败防枚举

- **GIVEN** 邮箱 `unknown@example.com` 未注册
- **WHEN** 用户提交登录表单
- **THEN**
  - 统一文案「邮箱或密码错误」（不提示「用户不存在」）
  - 表单密码框清空，邮箱保留

##### 异常：注册邮箱重复

- **GIVEN** `dev@example.com` 已注册
- **WHEN** 用户再次注册同一邮箱
- **THEN**
  - email 字段下方显示「该邮箱已注册」
  - 提供链接跳转 `/login?email=dev@example.com` 预填

## 6. 原型操作指南

在浏览器打开 `logos/resources/prd/2-product-design/2-page-design/core-03-auth-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认视图 | 登录表单 |
| 点击「创建账户」 | 切换注册视图 |
| 登录演示账号 `demo@coldrawdb.local` / `demo1234` | 进入「已登录」预览（含 user-menu + session 指示） |
| 点击「模拟 Token 过期」 | session 指示 amber → 续期成功 → 时间更新 |
| 点击「退出登录」 | 回到登录视图 |
| 切换 dark 主题 | `data-mode` 切换，表单对比度保持 |

## 7. 反模式（须避免）

- ❌ 将 refresh_token 存入 localStorage（XSS 风险）
- ❌ 登录失败区分「用户不存在 / 密码错误」（用户枚举）
- ❌ refresh 失败仍无限重试原请求（应单次 redirect login）
- ❌ 鉴权拦截阻断 `?share=` 只读链路（破坏 S02）
