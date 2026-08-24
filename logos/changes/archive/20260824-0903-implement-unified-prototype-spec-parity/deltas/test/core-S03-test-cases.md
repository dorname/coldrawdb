# Delta — core-S03-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

S03：注册 / 登录 / Token 续期 / 会话指示。成功后进入 **rooms**，不直达 editor。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（A 批）将页面与安全合同用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — 页面与安全合同

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S03-UI-01 | 未登录 | 打开默认入口 | `login-form`/`register-form` 可达；品牌区与双 tab 与主原型一致 | 本提案 A 批实现 |
| ST-S03-UI-02 | 登录页 | 错误密码 | 401；文案**不枚举**「邮箱是否存在」；无 token 原文 | 本提案 A 批实现 |
| ST-S03-UI-03 | 注册页 | 重复邮箱 | 409 字段/表单错误；不跳转 | 本提案 A 批实现 |
| ST-S03-UI-04 | 合法登录/注册 | 提交成功 | 进入 `rooms-list-page`；可见 `session-indicator` / 用户菜单 | 本提案 A 批实现 |
| ST-S03-UI-05 | access 过期 | 受保护 API | 单次 refresh 后重放；会话指示更新；失败则回登录 | 本提案 A 批实现 |
| UT-S03-ERR-01 | 错误 body | 解析 | UI 脱敏；不输出 cookie/token | 本提案 A 批实现 |

## MODIFIED — 既有 S03 用例补充约束

保留后端断言；补充：**前端成功路径必须以 rooms 为下一跳**；错误路径禁止用户枚举。本提案 A 批必须覆盖上表 UI 用例，不得仅以 API 200 视为「已对齐主原型」。
