# Delta — core-S03-user-auth-design.md（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（SaaS 鉴权层 + 编辑器入口） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（登录 / 注册 / 会话续期 → 房间 → 编辑器） |
| 历史参考 | `core-03-auth-prototype.html`（保留，不再作为验收入口） |
| 生产实现 | 后端 auth API/DB 已实现；`frontend-rs` 登录/注册/续期界面尚未接入 |
| 视觉基准 | 统一 token + Light/Dark 玻璃态设计系统 |
| 与 main 关系 | drawdb 无对应页面；交互模式参考常见 B2B SaaS（单栏表单 + 品牌侧栏） |

## MODIFIED — 6. 原型操作指南

在浏览器打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认视图 | 登录表单 |
| 点击「创建账户」 | 切换注册视图 |
| 输入任意合法邮箱与至少 8 位密码 | 进入房间列表（含 user-menu + session 指示） |
| 点击「模拟 Token 过期」 | session 指示 amber → 续期成功 → 时间更新 |
| 点击「退出登录」 | 回到登录视图 |
| 切换 dark 主题 | `data-mode` 切换，表单对比度保持 |

`core-03-auth-prototype.html` 只用于追溯早期方案，不要求修复其控件或测试锚点。

## ADDED — 9. 生产实现状态

统一主原型中的鉴权通过浏览器内状态机模拟，不发送真实 auth 请求。它证明交互设计完整，不等价于生产前端已实现；生产验收必须以 `frontend-rs` 代码、S03 API 编排和真实网络测试为准。

