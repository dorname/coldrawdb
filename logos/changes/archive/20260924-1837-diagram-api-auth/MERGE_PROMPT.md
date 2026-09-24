# 合并指令

## 变更提案
- 提案名称：diagram-api-auth
- 提案目录：logos/changes/diagram-api-auth/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/api/bridge.yaml

- Delta 文件：`logos/changes/diagram-api-auth/deltas/api/bridge.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/diagram-api-auth/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/diagram-api-auth/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/database/coldrawdb-v1.sql

- Delta 文件：`logos/changes/diagram-api-auth/deltas/database/coldrawdb-v1.sql`
- 目标目录：`logos/resources/database/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/scenario/core-S01-diagram-save.json

- Delta 文件：`logos/changes/diagram-api-auth/deltas/scenario/core-S01-diagram-save.json`
- 目标目录：`logos/resources/scenario/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/scenario/core-S02-shared-link-load.json

- Delta 文件：`logos/changes/diagram-api-auth/deltas/scenario/core-S02-shared-link-load.json`
- 目标目录：`logos/resources/scenario/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/test/core-S01-test-cases.md

- Delta 文件：`logos/changes/diagram-api-auth/deltas/test/core-S01-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/test/core-S02-test-cases.md

- Delta 文件：`logos/changes/diagram-api-auth/deltas/test/core-S02-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/diagram-api-auth/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(diagram-api-auth): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive diagram-api-auth`。
