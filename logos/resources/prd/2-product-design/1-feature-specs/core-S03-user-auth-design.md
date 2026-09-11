# S03：用户注册 / 登录 / Token 续期 — 交互设计

> 模块：core | 场景：S03 | 版本：V2 | 优先级：P2
> 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-03-auth-prototype.html`（不作为验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 `implement-unified-prototype-spec-parity`
> 成功后默认进入：**rooms**（不是直达 `/editor`）
> Phase 1 输入：`core-00-scenario-overview.md` §S03 / `core-01-requirements.md` US 关联 P03
> 参考：drawdb main **无用户鉴权**（V1 完全匿名）；本场景为 coldrawdb V2 net-new，为 S04/S05 前置

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 生产实现 | 后端 auth API/DB 已实现；生产前端已有部分鉴权接入；须对齐主原型：登录/注册表单、会话指示、退出确认、成功后进入 rooms、主题一致 |
| 主原型 | 唯一评审与规格事实输入；演示登录不发送真实请求 |

## 2. 信息架构（V2 增量）

### 2.1 路由

| 路由 / 状态 | 页面 | 未登录 | 已登录 |
|---|---|---|---|
| `/login` · `/register` 或同壳 `auth` | 登录/注册 | ✅ | 重定向 **`/rooms`** |
| `/rooms` | 房间列表 | 重定向登录 | ✅ |
| `/rooms/.../editor`（room-editor） | 协作编辑器 | 需登录 + 成员身份 | ✅ |
| `/?share=` | 分享只读 | ✅ 匿名只读（S02） | ✅ |

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

在浏览器打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认视图 | 登录表单 |
| 点击「创建账户」 | 切换注册视图 |
| 输入任意合法邮箱与至少 8 位密码 | 进入房间列表（含 user-menu + session 指示） |
| 点击「模拟 Token 过期」 | session 指示 amber → 续期成功 → 时间更新 |
| 点击「退出登录」 | 回到登录视图 |
| 切换 dark 主题 | `data-mode` 切换；登录/注册表单字段、字段标签、错误提示、auth-tabs 与主按钮在暗色玻璃背景下对比度 ≥ WCAG AA 4.5:1，输入框与 tab 边框使用 `--line-strong`（.30）清晰可辨 |

`core-03-auth-prototype.html` 只用于追溯早期方案，不要求修复其控件或测试锚点。
## 7. 反模式（须避免）

- ❌ 将 refresh_token 存入 localStorage（XSS 风险）
- ❌ 登录失败区分「用户不存在 / 密码错误」（用户枚举）
- ❌ refresh 失败仍无限重试原请求（应单次 redirect login）
- ❌ 鉴权拦截阻断 `?share=` 只读链路（破坏 S02）

## 8. 单文件鉴权演示

### 8.1 可演示状态

| 状态 | 触发 | 反馈 | 后续状态 |
|---|---|---|---|
| 登录校验失败 | 邮箱为空/格式错误或密码不足 8 位 | 字段描边、中文错误文案、错误摘要 | 留在登录页 |
| 登录成功 | 使用任意合法邮箱与 8 位密码 | 按钮 loading → Toast | 房间列表 |
| 注册 | 切换至注册并完成 display name、邮箱、密码、确认密码 | 密码强度与一致性实时反馈 | 房间列表 |
| 凭据失败 | 点击「模拟错误」后提交 | 通用「邮箱或密码错误」，不泄露邮箱是否存在 | 登录页 |
| Token 续期 | 编辑器用户菜单点击「模拟 Token 过期」 | 会话点转 amber → 续期成功 Toast | 编辑器，操作不中断 |
| 退出 | 用户菜单点击退出 | 有未保存操作时先确认；否则清理原型会话 | 登录页 |

### 8.2 连续体验约束

- 鉴权页和编辑器必须共享 Light/Dark 主题、玻璃卡片、按钮、输入框、焦点环和 Toast 语言。
- 登录成功不得刷新 HTML；状态 store 切换到房间视图，并保留当前主题。
- 登录表单不得把演示密码写入 localStorage；原型不发起真实请求。
- `data-testid="login-form"`、`login-submit`、`register-form`、`register-submit`、`user-menu`、`session-indicator` 必须保留。

### 8.3 无障碍与键盘

- 所有输入有显式 label；错误文本使用 `aria-describedby`，提交错误摘要使用 `role="alert"`。
- Enter 提交当前表单；密码显隐按钮有动态 `aria-label`。
- 视图切换后焦点进入主标题；loading 时按钮使用 `aria-busy="true"`。

## 9. 生产实现状态

统一主原型中的鉴权通过浏览器内状态机模拟，不发送真实 auth 请求。它证明交互设计完整，不等价于生产前端已逐项完成。

准确状态：

1. 后端 auth 已实现并可编排验收
2. 生产前端已有部分页面/API 接入
3. 相对主原型的表单、锚点、主题、会话续期与进入 rooms 的逐项对齐，以本规格为合同，由下一代码变更实现

`data-testid`：`login-form`、`login-submit`、`register-form`、`register-submit`、`user-menu`、`session-indicator` 必须保留。

## 注册/登录成功跳转

凡原文写「跳转 `/editor`」的成功路径，一律改为：**进入 rooms 视图**（`rooms-list-page`）。邮箱验证若启用，验证完成后再进入 rooms。
