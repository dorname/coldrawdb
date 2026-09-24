# Delta — core-smoke-test-cases.md（diagram-api-auth）

> 模块：core | 提案：diagram-api-auth
> 目标：smoke 运行在 `COLDRAWDB_DIAGRAMS_AUTH=on` 模式（runner 启动 host 服务时注入）；写操作前置 register/login 取 token；新增 SMOKE-core-08 匿名 401 断言。

## MODIFIED — 1. 范围

## 1. 范围

本文件覆盖 staging 环境的冒烟测试（smoke test）规格。

**执行时机**：每次部署到 staging 后由 `openlogos smoke` CLI 触发（人类确认点）

**失败处理**：任何 smoke 失败 → 阻断 release；回滚 staging

**对账**：与 `core-01-deployment-plan.md` §6 列出的 4 个 smoke 入口一致

**鉴权模式（diagram-api-auth）**：smoke runner 以 `COLDRAWDB_DIAGRAMS_AUTH=on` 启动 host 服务，冒烟在强制鉴权模式下执行。前置：`POST /api/v1/auth/register` + `POST /api/v1/auth/login` 获取 smoke token（专用 smoke 用户），所有写操作与登录态读操作注入 `Authorization: Bearer <smoke_token>`；SMOKE-core-01（健康检查）与静态资源用例保持匿名（不受 flag 影响）。

## MODIFIED — 4.1 目的

### 4.1 目的

验证 S01（编辑保存）+ S02（分享加载）主链路。**diagram-api-auth：全部步骤携带 `Authorization: Bearer <smoke_token>`（runner 前置 register/login 获取）；无 token 时本用例各步骤应返回 401（由 SMOKE-core-08 统一断言）。**

## MODIFIED — 5.1 目的

### 5.1 目的

验证 bridge I/O（S03 桥接 API 主链路）。**diagram-api-auth：bridge 端点同策略强制鉴权，请求携带 `Authorization: Bearer <smoke_token>`；匿名 401 由 SMOKE-core-08 统一断言。**

## MODIFIED — 8.6 SMOKE-core-07 — 导入缺 id payload 持久化验证（diagrams import 端点）

## 8.6 SMOKE-core-07 — 导入缺 id payload 持久化验证（diagrams import 端点）

### 目的

验证 `POST /api/v1/diagrams/import` 对缺 id 表/字段的 payload 不再静默丢数据（fix-diagram-import-persistence：假成功真丢数据 + name 覆盖为 NULL 的修复生效确认）。**diagram-api-auth：import 端点在 flag=on 下需 `Authorization: Bearer <smoke_token>`（runner 注入）；匿名 401 由 SMOKE-core-08 统一断言。**

### 步骤

1. POST `/api/v1/diagrams/import` body（携带 `Authorization: Bearer <smoke_token>`）：
   ```json
   {
     "source": "smoke",
     "payload": {
       "tables": [{"name": "smoke_no_id", "x": 0, "y": 0,
                   "fields": [{"name": "id", "type": "INT", "primary": true}]}],
       "references": []
     }
   }
   ```
   → 期望 200，`data.imported_tables == 1`、`data.imported_fields == 1`
2. GET `/api/v1/diagrams/{diagram_id}`（携带 smoke token）→ 期望 200
3. 清理：DELETE `/api/v1/diagrams/{diagram_id}`（携带 smoke token）→ 期望 200

### 断言

- `data.tables.length == 1` 且 `data.tables[0].id` 非空（服务端自动补全，非空字符串）
- `data.tables[0].fields.length == 1` 且 `data.tables[0].fields[0].id` 非空
- `data.name` 为字符串（payload 无 name 时兜底 `"imported_diagram"`，不得为 null）
- `data.revision >= 1`

### 失败处理

- `tables` 为空 → 持久化修复未生效，检查后端版本与镜像重建
- `name` 为 null → `save_diagram` 覆盖缺陷未修复
- 返回 401 → smoke token 获取/注入失败，检查 runner register/login 链路
- 返回其他 4xx/5xx → 检查后端日志（persist_import_payload 错误传播路径）

## ADDED — 8.7 SMOKE-core-08 — 强制鉴权 401 断言（diagram-api-auth）

### 目的

验证 `COLDRAWDB_DIAGRAMS_AUTH=on` 下匿名访问被拒：diagrams 写/读、bridge 端点无 token 一律 401，分享豁免仅认 share_token。

### 步骤

1. 匿名 `POST /api/v1/diagrams` → 期望 401
2. 匿名 `GET /api/v1/diagrams/{任意存在 id}`（无 share_token）→ 期望 401
3. 匿名 `POST /api/v1/bridge/import/local` → 期望 401
4. 匿名 `POST /api/v1/diagrams/{id}/share` → 期望 401（flag=on 下 share 铸造也需登录）
5. 带 smoke token `POST /api/v1/diagrams` → 期望 200（token 有效性反证）

### 断言

- 步骤 1-4 全部 401；步骤 5 为 200
- 401 响应体为 `{code, message, request_id}` envelope（code=401）

### 失败处理

- 任一步骤非 401/200 → 检查后端 `COLDRAWDB_DIAGRAMS_AUTH` 是否生效（runner 启动日志）与守卫中间件挂载
- 步骤 5 失败 → smoke token 获取链路（register/login）异常，检查 auth_v1

## MODIFIED — 附录 A：用例 ID 清单

| ID | 标题 |
|---|---|
| SMOKE-core-01 | 健康检查 |
| SMOKE-core-02 | 创建 + 读取 E2E |
| SMOKE-core-03 | 导入导出 E2E |
| SMOKE-core-04 | 静态资源加载 |
| SMOKE-core-05 | 数据库健康 |
| SMOKE-core-06 | 本地脚本启停验证 |
| SMOKE-core-STABLE-01 | 稳定版 Compose 健康检查 |
| SMOKE-core-07 | 导入缺 id payload 持久化验证（diagrams import 端点） |
| SMOKE-core-08 | 强制鉴权 401 断言（diagram-api-auth） |
