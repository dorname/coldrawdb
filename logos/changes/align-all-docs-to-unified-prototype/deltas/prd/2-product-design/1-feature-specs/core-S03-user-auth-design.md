# Delta — core-S03-user-auth-design.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 模块：core | 场景：S03 | 版本：V2 | 优先级：P0
> 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-03-auth-prototype.html`（不作为验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 `implement-unified-prototype-spec-parity`
> 成功后默认进入：**rooms**（不是直达 `/editor`）

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 生产实现 | 后端 auth API/DB 已实现；生产前端已有部分鉴权接入；须对齐主原型：登录/注册表单、会话指示、退出确认、成功后进入 rooms、主题一致 |
| 主原型 | 唯一评审与规格事实输入；演示登录不发送真实请求 |

## MODIFIED — 2.1 路由

### 2.1 路由

| 路由 / 状态 | 页面 | 未登录 | 已登录 |
|---|---|---|---|
| `/login` · `/register` 或同壳 `auth` | 登录/注册 | ✅ | 重定向 **`/rooms`** |
| `/rooms` | 房间列表 | 重定向登录 | ✅ |
| `/rooms/.../editor`（room-editor） | 协作编辑器 | 需登录 + 成员身份 | ✅ |
| `/?share=` | 分享只读 | ✅ 匿名只读（S02） | ✅ |

## ADDED — 统一原型对齐补充：注册/登录成功跳转

凡原文写「跳转 `/editor`」的成功路径，一律改为：**进入 rooms 视图**（`rooms-list-page`）。邮箱验证若启用，验证完成后再进入 rooms。

## MODIFIED — 9. 生产实现状态

统一主原型中的鉴权通过浏览器内状态机模拟，不发送真实 auth 请求。它证明交互设计完整，不等价于生产前端已逐项完成。

准确状态：

1. 后端 auth 已实现并可编排验收
2. 生产前端已有部分页面/API 接入
3. 相对主原型的表单、锚点、主题、会话续期与进入 rooms 的逐项对齐，以本规格为合同，由下一代码变更实现

`data-testid`：`login-form`、`login-submit`、`register-form`、`register-submit`、`user-menu`、`session-indicator` 必须保留。
