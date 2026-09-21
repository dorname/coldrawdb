# Delta — core-smoke-test-cases.md（修改）

> 模块：core | 提案：fix-diagram-import-persistence

## ADDED — SMOKE-core-07 — 导入缺 id payload 持久化验证（diagrams import 端点）

### 目的

验证 `POST /api/v1/diagrams/import` 对缺 id 表/字段的 payload 不再静默丢数据（fix-diagram-import-persistence：假成功真丢数据 + name 覆盖为 NULL 的修复生效确认）。

### 步骤

1. POST `/api/v1/diagrams/import` body：
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
2. GET `/api/v1/diagrams/{diagram_id}` → 期望 200
3. 清理：DELETE `/api/v1/diagrams/{diagram_id}` → 期望 200

### 断言

- `data.tables.length == 1` 且 `data.tables[0].id` 非空（服务端自动补全，非空字符串）
- `data.tables[0].fields.length == 1` 且 `data.tables[0].fields[0].id` 非空
- `data.name` 为字符串（payload 无 name 时兜底 `"imported_diagram"`，不得为 null）
- `data.revision >= 1`

### 失败处理

- `tables` 为空 → 持久化修复未生效，检查后端版本与镜像重建
- `name` 为 null → `save_diagram` 覆盖缺陷未修复
- 返回 4xx/5xx → 检查后端日志（persist_import_payload 错误传播路径）

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
| SMOKE-core-07 | 导入缺 id payload 持久化验证 |
