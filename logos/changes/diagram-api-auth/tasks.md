# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/api/diagrams.yaml` — 全部端点 401/403 语义 + `POST /diagrams/{id}/share` + `GET /diagrams/{id}?share_token=` 匿名豁免 + 开关行为说明
- [x] 产出 delta 文件到 `deltas/api/bridge.yaml` — 5 端点 401 语义（与 diagrams 同策略）
- [x] 产出 delta 文件到 `deltas/api/mcp-tools.yaml` — 全工具标注：后端强制模式下 `COLDRAWDB_ACCESS_TOKEN` 必填
- [x] 产出 delta 文件到 `deltas/database/` — diagram 表新增 `share_token VARCHAR`（可空）+ 迁移说明（存量 NULL 属预期）
- [x] 产出 delta 文件到 `deltas/scenario/core-S02-shared-link-load.json` — 分享链接携带 share_token 匿名加载时序
- [x] 产出 delta 文件到 `deltas/scenario/core-S01-diagram-save.json` — 编辑保存需登录态（Bearer JWT）
- [x] 产出 delta 文件到 `deltas/test/core-S01-test-cases.md` — 401 未授权 / 分享豁免 / share 铸造轮换用例
- [x] 产出 delta 文件到 `deltas/test/core-S02-test-cases.md` — 分享加载匿名可读 + 无 token 404/401 用例
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — smoke 前置改为 register/login 取 token；开启模式下 401 断言
- [x] **验证 API YAML** — `logos/resources/api/` 下所有文件必须为有效 YAML（含 `:` 或特殊字符的 description/summary 值必须双引号包裹）

## [code] 代码实现

- [ ] 实现代码变更

## [deploy] 部署任务

- [ ] 重建镜像并重启本地 compose 栈（开关保持默认 off，生产无感）
- [ ] 执行迁移：`ALTER TABLE diagram ADD COLUMN share_token VARCHAR`（幂等）
- [ ] 确认：off 模式行为不变；on 模式（临时开关验证后恢复 off）401 与分享豁免按规格生效
