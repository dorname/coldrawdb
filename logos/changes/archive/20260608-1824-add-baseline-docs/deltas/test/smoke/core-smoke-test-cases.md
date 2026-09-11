# ADDED — staging 冒烟测试用例规格
# 模块：core | 提案：add-baseline-docs
# 路径：`logos/resources/test/smoke/core-smoke-test-cases.md`
# 对齐参考源：`core-01-deployment-plan.md` §6 + `core-02-diagram-persistence.md` + `core-03-bridge-io.md`

## 1. 范围

本文件覆盖 staging 环境的冒烟测试（smoke test）规格。

**执行时机**：每次部署到 staging 后由 `openlogos smoke` CLI 触发（人类确认点）

**失败处理**：任何 smoke 失败 → 阻断 release；回滚 staging

**对账**：与 `core-01-deployment-plan.md` §6 列出的 4 个 smoke 入口一致

## 2. 用例 ID 前缀

`SMOKE-core-NN`（NN 从 01 开始）

## 3. SMOKE-core-01 — 健康检查

### 3.1 目的

验证 staging 后端进程存活 + 基础 API 可达

### 3.2 步骤

1. `GET /api/v1/diagrams/health` → 期望 200
2. 验证响应 body `{"status": "ok"}`
3. 验证响应时间 < 500ms

### 3.3 断言

- `status_code == 200`
- `response.status == "ok"`
- `response.duration < 500ms`

### 3.4 失败处理

- 503 / 超时 → 检查后端进程 + 日志
- 200 但 body 异常 → 检查后端版本

## 4. SMOKE-core-02 — 创建 + 读取 E2E

### 4.1 目的

验证 S01（编辑保存）+ S02（分享加载）主链路

### 4.2 步骤

1. POST `/api/v1/diagrams` body 含 1 张表 + 2 字段 → 期望 201，验证 `id` 非空、`revision: 0`
2. GET `/api/v1/diagrams/{id}` → 期望 200，验证 `tables.length == 1`、`fields.length == 2`
3. PUT `/api/v1/diagrams/{id}` body 包含 `revision: 0` + 新增 1 字段 → 期望 200，验证 `revision: 1`
4. GET 再次 → 期望 `fields.length == 3`
5. DELETE `/api/v1/diagrams/{id}` → 期望 204
6. GET 再次 → 期望 404

### 4.3 断言

- 6 个步骤全部通过
- 数据库中 `diagram` / `table` / `field` 表行数符合预期
- DELETE 后级联清理（直接查 SQL 验证）

### 4.4 失败处理

- 任一步骤失败 → 阻断 release
- 收集 `backend/logs/coldrawdb.log` 供排障

## 5. SMOKE-core-03 — 导入导出 E2E

### 5.1 目的

验证 bridge I/O（S03 桥接 API 主链路）

### 5.2 步骤

1. POST `/api/v1/bridge/import/local` body:
   ```json
   {
     "format": "sql",
     "engine": "mysql",
     "content": "CREATE TABLE smoke_users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL);",
     "title": "Smoke Import"
   }
   ```
   → 期望 201，验证 `task_id` 非空、`status: "pending"`

2. GET `/api/v1/bridge/import/local/logs?limit=1` → 期望 200，验证 `tasks[0].id == task_id` 且 `status` 已更新

3. POST `/api/v1/bridge/import/local/retry/{task_id}` （仅在 status=failed 时）→ 期望 200

4. 等待 task.status 变 `success`（最多 30s 轮询）
5. GET `/api/v1/diagrams/{task.diagramId}` → 期望 200，验证 `tables[0].name == "smoke_users"`

### 5.3 断言

- 5 个步骤全部通过
- SQL 解析正确（不含语法错误）
- Diagram 树结构与 SQL 一致

### 5.4 失败处理

- 解析失败 → 检查 SQL 兼容性（5.x 引擎子集）
- task 长时间 pending → 检查后端 worker 状态

## 6. SMOKE-core-04 — 静态资源加载

### 6.1 目的

验证 WASM 静态资源可达 + 入口 HTML 正确

### 6.2 步骤

1. GET `/` → 期望 200，验证 body 含 `<div id="root">` 与 `<script type="module">`
2. GET `/index.html` → 期望 200
3. GET `/trunk-bundle.js` （实际文件名由 trunk 决定）→ 期望 200，Content-Type: application/javascript
4. GET `/coldrawdb_wasm_bg.wasm` → 期望 200，Content-Type: application/wasm

### 6.3 断言

- 4 个 GET 全部 200
- Content-Type 正确
- WASM 文件 size > 100 KB（防止空文件）

### 6.4 失败处理

- 静态资源 404 → 检查 nginx 配置
- WASM 大小异常 → 检查 trunk 构建产物

## 7. SMOKE-core-05 — 数据库健康

### 7.1 目的

验证 SQLite 文件可读写 + 11 张表存在

### 7.2 步骤

1. POST `/api/v1/diagrams` body `{"title": "DB Health", "tables": []}` → 期望 201
2. 直接查 staging 容器内 SQLite：
   ```sql
   SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;
   ```
   验证返回 11 张业务表（不含 `sqlite_sequence`）
3. 直接 `INSERT INTO diagram VALUES (...)` + 验证可读
4. DELETE 测试用 diagram

### 7.3 断言

- 11 张表名匹配预期清单：
  `area, diagram, diagram_link, field, indice, indice_link, note, reference, table, table_link, task`
- 数据库文件可读写
- 磁盘空间 > 100 MB（健康阈值）

### 7.4 失败处理

- 表缺失 → 重新执行 `init.sql`
- 磁盘不足 → 清理备份

## 8. 通用要求

| 维度 | 要求 |
|---|---|
| 执行方式 | `openlogos smoke` CLI（人类确认点） |
| 总耗时 | < 60s |
| 失败阈值 | 任一失败 → 阻断 |
| 报告输出 | `logos/spec/smoke-report.md`（含每条用例状态） |
| 重试策略 | 网络错误自动重试 1 次；其他错误不重试 |
| 前置条件 | staging 已部署 + 后端进程存活 + 数据库可达 |

## 9. SMOKE 报告示例

```markdown
# Staging Smoke Report — 2026-06-08 10:00:00

## 总体
- 5/5 PASSED
- 总耗时 23.4s

## 详情

| ID | 状态 | 耗时 | 备注 |
|---|---|---|---|
| SMOKE-core-01 | ✅ PASS | 0.2s | health check |
| SMOKE-core-02 | ✅ PASS | 3.1s | create/read/update/delete |
| SMOKE-core-03 | ✅ PASS | 12.5s | SQL import → diagram |
| SMOKE-core-04 | ✅ PASS | 0.8s | 静态资源 4/4 |
| SMOKE-core-05 | ✅ PASS | 6.8s | 11 表齐全 |

## SMOKE_PASS
```

## 10. V1 边界

- ❌ 完整功能回归（V1 仅 smoke 5 项；完整回归在 UT/ST 阶段）
- ❌ 性能压测（V1 smoke 仅功能）
- ❌ 跨 staging 多实例（V1 单 staging）

## 11. 对齐参考源

- `core-01-deployment-plan.md`（smoke 入口）
- `core-02-diagram-persistence.md`（API 端点）
- `core-03-bridge-io.md`（bridge 端点）
- `core-S01-edit-and-save-diagram.md`（S01 时序）
- `core-S02-load-shared-diagram.md`（S02 时序）
- `backend/init.sql`（11 张表对账）
- `logos/spec/smoke-report.md`（报告格式）
- `logos/skills/deployment-executor/SKILL.md`（smoke 执行）
