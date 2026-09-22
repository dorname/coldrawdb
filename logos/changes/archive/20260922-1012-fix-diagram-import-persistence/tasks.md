# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/api/diagrams.yaml` — import 端点响应语义：imported_tables/imported_fields 按实际持久化数量上报、持久化失败返回 5xx、warnings 细化丢弃明细
- [x] 产出 delta 文件到 `deltas/api/mcp-tools.yaml` — `Diagram.name` 类型放宽为 `[string, "null"]` 并注明防御原因
- [x] 产出 delta 文件到 `deltas/test/core-S06-test-cases.md` — 新增导入回归用例（缺 id payload 导入后 GET 结构校验、name 兜底、虚报消除）
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — 新增部署后导入→GET 冒烟用例
- [x] **验证 API YAML** — `logos/resources/api/` 下所有文件必须为有效 YAML（所有包含 `:` 或特殊字符的 description/summary 值必须用双引号包裹）

## [code] 代码实现

- [x] 实现代码变更（backend 导入持久化修复 + mcp adapter 规范化 + 契约 name 可空，81+57 测试绿）

## [deploy] 部署任务

- [x] 重建并重启本地后端容器（docker compose up -d；镜像 `coldrawdb:v1` 重建于修复后，功能实测通过）
- [x] 执行存量数据迁移：`UPDATE diagram SET name='imported_diagram' WHERE name IS NULL`（幂等，affected_rows=0）
- [x] 确认迁移、服务启动正常（coldrawdb Healthy；nginx 9080 health=200、SPA=200；smoke Gate 3.8 PASS 8/8）
