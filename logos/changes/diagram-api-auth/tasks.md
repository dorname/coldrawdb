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

- [x] backend `COLDRAWDB_DIAGRAMS_AUTH` 开关（`auth::diagrams_auth_required()`，默认 off；测试经 `Option<web::Data<DiagramsAuthFlag>>` app_data 覆盖避免 env 竞态）+ main.rs 启动日志明示
- [x] diagrams_v1 5 端点 + share 端点接入守卫（`guard_diagrams_auth`）；GET /diagrams/{id} share_token 匿名豁免读（不匹配/未开分享/软删 → 404，无 token 无 share_token → 401）
- [x] `POST /api/v1/diagrams/{id}/share` 铸造/轮换（uuid v4 simple，重复调用即轮换；软删/不存在 → 404）
- [x] phase3_bridge 5 个未保护操作（import/local、logs、retry、config get/put）接入开关守卫；connect/export 维持既有强制鉴权（feat-db-connect-import-and-ddl-realdb-verify 现状，规格 off=现状 语义内）
- [x] 迁移 `backend/migrations/0011_diagram_share_token.up.sql`（+ down）；init.sql 不加列（apply_migrations 后执行，双份加列撞 duplicate column；新库经迁移带列）
- [x] mcp-server 启动期探测：无 COLDRAWDB_ACCESS_TOKEN 且后端 flag=on（匿名探针 401）→ CONFIG_INVALID 明确报错退出
- [x] smoke runner：`COLDRAWDB_DIAGRAMS_AUTH=on` 注入 + register/login 取 smoke token + 8 处 Bearer 注入 + Stage 6c SMOKE-core-08 匿名 401 断言（含 token 反证与清理）
- [x] UT/ST 测试（全部通过并写入 test-results.jsonl）：UT-S01-AUTH-01~05、UT-S02-10~13、ST-S01-AUTH-01、ST-S02-07（ST 落位 `diagrams_v1.rs::tests`，附录对齐路径已同步修正）
- [x] 附录 A 对齐路径修正：ST-S01-AUTH-01 / ST-S02-07 → `backend/src/diagrams_v1.rs::tests`（backend 为纯 bin crate，tests/ 无法引用内部模块）

## [deploy] 部署任务

- [ ] 重建镜像并重启本地 compose 栈（开关保持默认 off，生产无感）
- [ ] 执行迁移：`ALTER TABLE diagram ADD COLUMN share_token VARCHAR`（随 0011 迁移由后端启动自动应用，schema_migrations 记账幂等）
- [ ] 确认：off 模式行为不变；on 模式（临时开关验证后恢复 off）401 与分享豁免按规格生效
