# 变更提案：diagrams/bridge API 强制鉴权（分享链接匿名可读 + 开关过渡）

## 变更原因

当前 `diagrams_v1` 与 `bridge` 共 10 个端点零鉴权（任何能访问服务的人可读写全部图数据），这是部署方案中明确的演进阻塞点：「只有 diagram API 强制 S03 鉴权、完成细粒度授权和安全评审后，才可另案增加 Streamable HTTP」（MCP 整合到后端的前置）。rooms/collab 已强制 Bearer JWT，diagrams 成为最后一块无保护面。本提案为后续「后端 /mcp HTTP 端点」与「细粒度授权」铺路。

## 变更类型

接口级变更（API 语义 + DB schema + 代码 + 测试 + 部署）

## 变更范围

- 影响的 API：
  - `logos/resources/api/diagrams.yaml` — 全部端点补充 401/403 语义；新增 `POST /diagrams/{id}/share`（铸造/轮换 share_token，需登录）；`GET /diagrams/{id}?share_token=xxx` 匿名可读（豁免）；鉴权开关行为说明
  - `logos/resources/api/bridge.yaml` — 5 端点补充 401 语义（与 diagrams 同策略）
  - `logos/resources/api/mcp-tools.yaml` — 全工具标注：后端开启强制鉴权后 `COLDRAWDB_ACCESS_TOKEN` 必填
  - `logos/resources/api/auth.yaml` — 无需变更（register/login 既有），仅交叉引用
- 影响的 DB 表：`diagram` — 新增 `share_token VARCHAR`（可空）；迁移：存量图 `share_token IS NULL`（无分享链接，匿名不可读，属预期）
- 影响的业务场景：`core-S02-shared-link-load.json`（分享链接携带 share_token 匿名加载）、`core-S01-diagram-save.json`（编辑保存需登录态）
- 影响的编排/测试用例：`logos/resources/test/core-S01-test-cases.md`、`core-S02-test-cases.md`（401/分享豁免用例）、`logos/resources/test/smoke/core-smoke-test-cases.md`（smoke 前置改为 register/login 取 token）
- 影响的代码：
  - `backend/src/diagrams_v1.rs` — JWT 守卫 + share 端点 + share_token 豁免读
  - `backend/src/phase3_bridge.rs`（bridge 端点守卫）
  - `backend` 配置 — 新增 `COLDRAWDB_DIAGRAMS_AUTH`（默认 `off` = 现状匿名；`on` = 强制）
  - `backend/init.sql` — diagram 表加列
  - `mcp-server` — 启动期校验：后端强制模式且无 token → 明确错误提示
  - `scripts/smoke-local-scripts.sh` — register/login 获取 smoke token，请求注入 `Authorization: Bearer`
- **明确不含**（后续提案）：细粒度所有权授权（owner_id / 图级成员）、前端登录接线（编辑器登录态，完成后开关默认转 on）、后端 `/mcp` HTTP 端点（安全评审后另案）

## 部署影响

- 是否需要部署：是
- 部署原因：DB 加列 + 后端新守卫逻辑需重建镜像生效；开关默认 off，生产行为无变化，测试/冒烟在 on 模式验证
- 影响环境：本地 / 测试
- 是否涉及数据迁移：是（`diagram` 加 `share_token` 列，存量为 NULL，幂等）
- 是否需要回滚预案：是（回滚 = `COLDRAWDB_DIAGRAMS_AUTH=off`，列保留无害）
- 是否需要 smoke：是（开启模式下验证 401 与分享豁免全链路）

## 变更概述

为 diagrams/bridge 全部端点加上 Bearer JWT 强制鉴权（复用 auth_v1 既有体系），唯一匿名豁免是持 `share_token` 的单图读取（保住 S02 分享加载场景）；新增 `POST /diagrams/{id}/share` 铸造/轮换分享令牌。鉴权由 `COLDRAWDB_DIAGRAMS_AUTH` 开关控制，默认 off 保持现状匿名，前端登录接线完成后转默认 on——本提案只做后端能力与开关，不动生产前端。DB 侧 diagram 表加 `share_token` 可空列。smoke 与测试账本相应前置 register/login。
