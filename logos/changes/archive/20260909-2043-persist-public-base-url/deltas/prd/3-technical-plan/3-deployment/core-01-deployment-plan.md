## MODIFIED — 12.4 本地 dev 双进程启动（start-local.sh 环境变量约定）

### 12.4 本地 dev 双进程启动（start-local.sh 环境变量约定）

> 新增于 fix-local-start-and-invite-config：修复 start-local.sh 的 env 参数顺序缺陷（trunk 未启动）并接线 dev 覆盖口。
> 更新于 persist-public-base-url：`PUBLIC_BASE_URL` 由"空（需手动设置）"改为脚本默认注入前端入口，修复 dev 下邀请链接落到后端 `:3000` 导致 404 的缺陷（2026-09-09 实测复现）。

`./scripts/start-local.sh` 是本地 dev 双进程入口（backend `:3000` + trunk `:8080`），其环境变量约定：

| 变量 | 默认 | 说明 |
|---|---|---|
| `COLDRAWDB_BACKEND_PORT` | `3000` | 后端监听端口 |
| `COLDRAWDB_FRONTEND_PORT` | `8080` | trunk serve 端口 |
| `COLDRAWDB_API_BASE`（脚本内部接线） | `http://127.0.0.1:${COLDRAWDB_BACKEND_PORT}` | 前端编译期 dev 覆盖口（§12.2）：双进程下前端 base_url 同源派生会指向 trunk `:8080`（无代理），必须由脚本在 `trunk serve` 前注入该编译期变量使 API 直连后端。**生产单入口构建不得携带该变量**（§12.2 约束不变） |
| `PUBLIC_BASE_URL` | `http://127.0.0.1:${COLDRAWDB_FRONTEND_PORT}`（脚本默认注入） | dev 下邀请链接基址，默认指向**前端入口**（trunk 端口）。跨机验证时显式覆盖为局域网地址（如 `PUBLIC_BASE_URL=http://192.168.1.10:8080 ./scripts/start-local.sh`）；显式设置优先于默认值 |

**dev 下 `PUBLIC_BASE_URL` 必须指向前端入口的原因**：双进程无代理时 API 请求直达后端 `:3000`，若缺省走 Host 推导（§12.1 规则 2），生成的 `invite_url` 会以 `:3000` 为基址；而 SPA 页面由 trunk `:8080` 服务，被邀请人打开 `:3000` 的 `/invite/{token}` 得不到页面。生产单入口（§12.3）无此问题——前后端同源，Host 推导天然正确。

**`env` 调用约束**：脚本中 `env` 的选项（如 `-u FORCE_COLOR`）必须放在 `NAME=VALUE` 赋值之前（GNU env 参数顺序），否则 `-u` 会被当作命令名导致子进程未启动（本提案修复的缺陷，fix-local-start-and-invite-config）。
