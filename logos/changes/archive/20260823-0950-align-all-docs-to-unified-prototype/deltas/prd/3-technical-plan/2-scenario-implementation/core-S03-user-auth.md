# Delta — core-S03-user-auth.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 版本：V2 | 优先级：P2 | 前置：无 | 后续：S04
> Phase 2：`core-S03-user-auth-design.md` | 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-03-auth-prototype.html`（非验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity`
> **成功后默认进入：`rooms`（不是直达 `/editor`）**
> API/DB：预期不新增；仅补前端参与者与跳转/异常映射

## MODIFIED — 2. 参与者

| 角色 | 模块 | 说明 |
|---|---|---|
| AuthUI | `frontend-rs` auth 壳 | `auth` 页面状态；`login-form` / `register-form`；主题与编辑器一致 |
| AuthClient | `frontend-rs` | 真实 REST；access 内存；refresh mutex；`credentials: include` |
| AuthAPI 等 | backend `auth_*` | 既有 5 端点（register/login/refresh/logout/me） |
| 主原型演示器 | HTML 本地状态机 | **禁止**当作生产 HTTP 完成证据 |

## ADDED — 统一原型对齐补充：3 / 4 成功跳转（时序文案）

凡 Step 写「跳转 `/editor`」或「redirect … `/editor`」处，一律改为：

- 默认：进入 **`rooms`**（`rooms-list-page`）
- 若存在 `?redirect=`：仅允许指向工作空间内路径（如 `/rooms`、`/invite/...`、room-editor）；**禁止**默认落到历史空白编辑器
- 邮箱验证启用时：验证完成后再进入 rooms

对应修改点：

- §3 注册时序 Note / Step 19：`跳转 /editor 或 /verify-email` → `跳转 rooms 或 /verify-email`
- §4 登录时序 Step 20：`redirect query redirect or /editor` → `redirect query 或 rooms`
- §7.2 Step 1 示例：`/login?redirect=/editor/d-abc` → `/login?redirect=/rooms/...` 或 invite/room-editor
- §7.2 Step 8：按 redirect 或 **rooms**
- EX-18.1：未验证前私有 API 仍 401；验证后进 rooms（非 `/editor` 直达）

## ADDED — 会话反馈锚点（前端）

| 事件 | UI | `data-testid` |
|---|---|---|
| 登录/注册表单 | auth 壳 | `login-form` / `register-form` / `*-submit` |
| 会话有效 | 用户菜单 / 指示 | `user-menu` / `session-indicator` |
| refresh 失败 | Toast「登录已过期」→ auth | — |
| 退出 | Confirm（若 dirty）→ `/login` | `user-menu` |

## ADDED — 异常映射（前端，补齐）

| 代码 | 前端行为 |
|---|---|
| 201 / 200 登录成功 | 存 access → 导航 **rooms** |
| 409 EMAIL_EXISTS | inline 错误，停留注册 |
| 401 INVALID_CREDENTIALS | 统一文案，不泄露枚举 |
| 429 TOO_MANY_ATTEMPTS | 禁用提交 + 倒计时 |
| 401 REFRESH_INVALID | 清 session → auth（可带 redirect） |
| token_expired（业务 API） | 静默 refresh 后重放（见 §5） |

## ADDED — 统一原型对齐补充：13 / 14 衔接

- ✅ 登录成功 → **rooms** →（S04）room-editor →（S05）WS
- ✅ `?share=` 仍匿名（S02），不经本场景阻断
- ❌ 成功后直达空白 `/editor`（废止）

## MODIFIED — 15. 对齐参考源

- 现行主原型：`core-01-editor-prototype.html`（替代仅依赖 `core-03-auth-prototype.html` 作为验收）
- `core-00-information-architecture.md` — `auth` → `rooms`
