# Delta — core-S03-user-auth-design.md（修改）

> module: core | proposal: improve-unified-collab-prototype | 2026-08-18
> merge 时修改原型引用，并将下列章节追加到正式规格。

## MODIFIED — 1. 产品类型与原型策略

> 替换主文档 `core-S03-user-auth-design.md` §1 整节；顶部元数据中的原型路径作为阶段历史记录保留。

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（SaaS 鉴权层 + 编辑器入口） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（登录 / 注册 / 会话续期 → 房间 → 编辑器） |
| 历史参考 | `core-03-auth-prototype.html`（保留，不再作为验收入口） |
| 视觉基准 | 统一 token + Light/Dark 玻璃态设计系统 |
| 与 main 关系 | drawdb 无对应页面；交互模式参考常见 B2B SaaS（单栏表单 + 品牌侧栏） |

## ADDED — §8 单文件鉴权演示

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
