# Delta — core-01-deployment-plan.md（#1 compose 注入 PUBLIC_BASE_URL）

> 变更：fix-remote-github-issues / GitHub #1  
> 语义重申：邀请链接基址 = **公开 SPA 入口**（nginx:80 / 单入口域名），**不是**裸后端 `:3000` 调试口。历史缺陷曾把 Host 推到 `:3000` 导致无 SPA；禁止回退。

## MODIFIED — §5.1 Docker Compose 环境变量

在 `coldrawdb` service 的 `environment` 中 **ADDED**：

```yaml
PUBLIC_BASE_URL: "${PUBLIC_BASE_URL:-http://localhost}"
```

约定：

| 变量 | 默认 | 说明 |
|---|---|---|
| `PUBLIC_BASE_URL` | `http://localhost`（compose 默认；对应 nginx 映射 `:80`） | 后端生成 `invite_url` 的公开基址。跨机/域名部署时显式覆盖，如 `PUBLIC_BASE_URL=http://192.168.20.106` 或 `https://coldrawdb.example.com` |
| （既有）Host 推导回退 | — | 未设置或空字符串时仍按 §12.1 规则 2 从请求 Host + `X-Forwarded-Proto` 推导 |

**禁止**：把默认值设为 `http://localhost:3000` 或仅后端端口——被邀请人打开后可能拿不到 SPA（取决于是否直连后端静态服务；公开入口一律走 nginx:80 / 同源域名）。

## MODIFIED — §12.1 / 验收清单

- 补充勾选项：staging `docker compose` 启动后，`POST /rooms/{id}/invites` 返回的 `inviteUrl` 前缀等于已配置的 `PUBLIC_BASE_URL`（或未配置时等于浏览器访问的公开 Host），路径为 `/invite/{token}`。
- 明确：issue 截图形态 `http://{lan-ip}/invite/...`（无 `:3000`）在「经 nginx:80 访问」下为**正确**；若链接不可达，应改 `PUBLIC_BASE_URL` 为真实可达入口，而非改成 `:3000`。

## ADDED — §12.5 staging compose 与 PUBLIC_BASE_URL

> 更新于 fix-remote-github-issues：staging compose 默认注入公开入口基址，修复「未配置时完全依赖 Host、跨机分享易踩坑」的操作缺口。

1. `docker-compose.yml` 的 `coldrawdb.environment` 含 `PUBLIC_BASE_URL`（可被 shell 环境覆盖）。
2. 文档示例：`PUBLIC_BASE_URL=http://192.168.20.106 docker compose up -d`。
3. 与 §12.4 dev 脚本默认指向 trunk `:8080` 并列：dev → 前端端口；staging/prod → 公开 HTTP(S) 入口。
