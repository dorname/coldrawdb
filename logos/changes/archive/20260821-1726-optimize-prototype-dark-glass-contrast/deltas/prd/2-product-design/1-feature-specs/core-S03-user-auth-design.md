# Delta — core-S03-user-auth-design.md（修改）

> module: core | proposal: optimize-prototype-dark-glass-contrast
> 仅更新色彩 / 字体 / 对比度说明，不改变任何交互语义、路由、状态机与测试锚点。
> 色板与组件覆盖规则的实现载体为主原型 `core-01-editor-prototype.html`，对应 delta：`deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（SaaS 鉴权层 + 编辑器入口） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（登录 / 注册 / 会话续期 → 房间 → 编辑器） |
| 历史参考 | `core-03-auth-prototype.html`（保留，不再作为验收入口） |
| 生产实现 | 后端 auth API/DB 已实现；`frontend-rs` 登录/注册/续期界面尚未接入 |
| 视觉基准 | 统一 token + Light/Dark 玻璃态设计系统；Dark 模式采用高对比度暗色色板（以主原型 `html[data-mode="dark"]` token 组为准：`--bg:#050f13`、`--surface` 不透明度 .86、文字层级 `--text:#f2fdfe` / `--text-2:#b8d2d8` / `--text-3:#86a3ab`），正文、标签与错误提示文字对背景对比度均 ≥ WCAG AA 4.5:1 |
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
| 切换 dark 主题 | `data-mode` 切换；登录/注册表单字段、字段标签、错误提示、auth-tabs 与主按钮在暗色玻璃背景下对比度 ≥ WCAG AA 4.5:1，输入框与 tab 边框使用 `--line-strong`（.30）清晰可辨 |

`core-03-auth-prototype.html` 只用于追溯早期方案，不要求修复其控件或测试锚点。
