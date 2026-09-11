# Delta: prd/2-product-design/1-feature-specs/core-03-bridge-io.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## ADDED — 14. 连接数据库导出执行（V2 增量）

> 提案：feat-room-recycle-bin-dropdown-and-db-export

### 14.1 端点契约

`POST /api/v1/bridge/export/execute`，挂既有 auth 中间件（Bearer）。

请求：

```json
{ "engine": "sqlite | postgres", "source": "<SQLite 文件绝对路径 | PG 连接串>", "ddl": "CREATE TABLE ..." }
```

响应 200（`ApiResp` 包裹）：

```json
{ "code": 0, "data": { "engine": "sqlite", "statements": 5, "tables": 3 } }
```

- `ddl`：通常为前端导出预览（`export_diagram_sql`）生成的 SQL 文本；后端不做方言改写，原样执行。
- `statements`：成功执行的语句条数（按分号拆分逐条执行）。
- `tables`：所执行 DDL 中 `CREATE TABLE` 语句计数（大小写不敏感，含 `IF NOT EXISTS` 变体）。
- 任一条语句执行失败即中止，不继续执行后续语句（不重试、不跳过）。

错误语义：

| 状态码 | 场景 | 前端文案 |
|---|---|---|
| 400 | `engine` 非 `sqlite`/`postgres`，或 `source`/`ddl` 为空 | 「不支持的数据库引擎」/「请填写连接信息」 |
| 502 | 连接失败（文件不可写 / PG 不可达 / 认证失败）或 DDL 执行失败（语法错误 / 表已存在 / 约束冲突） | 「导出执行失败：<引擎错误消息>」 |

### 14.2 执行口径

- **SQLite**（sqlx-sqlite）：以写模式打开（不存在则创建文件）；逐条 `execute` 拆分后的语句。
- **PostgreSQL**（sqlx-postgres）：单连接逐条 `execute`；不开显式事务（DDL 在 PG 可事务化，但为与 SQLite 口径一致、且便于定位失败语句，采用逐条中止策略）。
- 语句拆分：按 `;` 拆分并忽略空串/纯注释段（与前端导出预览生成的语句边界一致；不解析存储过程等含内嵌分号的语法——超出当前导出器产出范围）。
- 幂等性由调用方负责：导出器默认生成不含 `IF NOT EXISTS` 的 DDL（postgresql/generic），重复执行会在「表已存在」处 502，属预期行为并在错误消息中体现。

### 14.3 安全边界

- 与 §13.4 一致：`source` 仅由后端进程本机解析，不在前端持久化、不写导入日志表（连接串可能含口令）。
- 端点需登录态；不校验路径白名单（桌面/自托管场景，服务端即用户本机）。
- `ddl` 原样执行、不做内容白名单——与「用户对自己本机/自有库执行自己生成的 DDL」的工具定位一致；该端点不面向多租户共享部署。
