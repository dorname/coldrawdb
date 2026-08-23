# Delta — core-S03-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

S03：注册 / 登录 / Token 续期 / 会话指示。成功后进入 **rooms**，不直达 editor。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED — 页面与安全合同

| ID | 前置 | 操作 | 预期 | 变更 |
|---|---|---|---|---|
| ST-S03-UI-01 | 未登录 | 打开默认入口 | `login-form`/`register-form` 可达；品牌区与双 tab 与主原型一致 | ADDED |
| ST-S03-UI-02 | 登录页 | 错误密码 | 401；文案**不枚举**「邮箱是否存在」；无 token 原文 | ADDED |
| ST-S03-UI-03 | 注册页 | 重复邮箱 | 409 字段/表单错误；不跳转 | ADDED |
| ST-S03-UI-04 | 合法登录/注册 | 提交成功 | 进入 `rooms-list-page`；可见 `session-indicator` / 用户菜单 | ADDED |
| ST-S03-UI-05 | access 过期 | 受保护 API | 单次 refresh 后重放；会话指示更新；失败则回登录 | ADDED |
| UT-S03-ERR-01 | 错误 body | 解析 | UI 脱敏；不输出 cookie/token | ADDED |

## ADDED — 统一原型对齐补充：既有 UT-S03-01～07 / ST-S03-01

保留后端断言；补充：**前端成功路径必须以 rooms 为下一跳**；错误路径禁止用户枚举。
