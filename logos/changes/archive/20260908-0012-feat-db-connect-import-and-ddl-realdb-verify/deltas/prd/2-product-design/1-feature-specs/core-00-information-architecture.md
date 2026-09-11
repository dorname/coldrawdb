# Delta: core-00-information-architecture.md

> 提案：feat-db-connect-import-and-ddl-realdb-verify

## MODIFIED — 9.3 Phase 边界更新

替换为（导入来源新增「数据库连接」能力登记）：

```markdown
### 9.3 Phase 边界更新

| 能力 | Phase C |
|------|---------|
| 导入/导出侧边抽屉 | ✅ |
| SQL/DBML 全屏视图 | ❌（Phase D） |
| 连接数据库导入（SQLite/PG introspection → 本地合并） | ✅（feat-db-connect-import-and-ddl-realdb-verify，见 core-03 §11 / core-01d §4.4） |
```

## MODIFIED — 8. V2 增量：IO 抽屉（Phase C）

节首引言块后追加 IA 注记（其余内容不变）：

```markdown
> 导入来源（feat-db-connect-import-and-ddl-realdb-verify）：ImportDrawer 格式 Tab 由 SQL/DBML/JSON 扩展为 SQL/DBML/JSON/数据库；「数据库」来源经 `POST /api/v1/bridge/import/connect` introspect 真实库返回 DDL，合入画布复用本地合并管道，不产生新页面/新路由。
```
