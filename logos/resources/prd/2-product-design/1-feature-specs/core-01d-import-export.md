# 导入 / 导出 IO 抽屉规格（V2 / Phase C）

## 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。IO 入口与格式以主原型「更多菜单 → 导入/导出抽屉」为准。

| 项 | 约定 |
|---|---|
| 页面流 | IO 仅在 `room-editor`（及 EmptyGuide 引导）触发；不依赖独立历史原型 |
| 演示 ≠ 生产 | 主原型导入为本地解析模拟；剪贴板/下载以 Toast 或本地 blob 演示。生产导入走 bridge API，导出客户端生成 |
| 实现状态 | bridge / 导出能力**后端或客户端已具备**；**生产前端部分接入**；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity` |

## 1. 概述

Phase C 将 V1 居中 **Import 模态** 与占位 **Export 按钮** 升级为画布右侧 **非模态 IO 抽屉**：

- 不占用 `ModalRoot` 遮罩层（L4），与 Inspector（L3）同级侧栏语义
- 导入走 `POST /api/v1/bridge/import/local`
- 导出在**客户端**从当前 `EditorStore` 生成 SQL / DBML / JSON 预览（V1 无服务端 export 端点）

## 2. 组件树

```
AppRoot
├── AppBar
│   └── 更多菜单 (btn-more-menu)
│         ├── btn-import  → ImportDrawer
│         └── btn-export  → ExportDrawer
├── ToolRail / Command Palette（可命令打开同一抽屉）
├── IoDrawer / SideSheet
│   ├── ImportDrawer  data-testid="import-drawer"
│   └── ExportDrawer  data-testid="export-drawer"
└── EmptyGuide → guide-import-sql → ImportDrawer
```

**主路径变更**：导入/导出**不再**以 AppBar 常驻 pill 为默认；统一经 **更多菜单**（与主原型 `renderMoreMenu` 一致）。`btn-import` / `btn-export` testid 保留在菜单项上。

## 3. 状态模型

```rust
enum IoDrawerKind {
    None,
    Import,
    Export,
}
```

| 信号 | 类型 | 说明 |
|------|------|------|
| `io_drawer` | `RwSignal<IoDrawerKind>` | 当前打开的抽屉 |
| `inspector_open_before_io` | 可选缓存 | 打开 IO 抽屉前 Inspector 是否展开，关闭时恢复 |

### 3.1 互斥规则

- 打开 Import / Export 抽屉时折叠 Inspector（`data-testid="inspector"`），关闭时恢复。
- 与成员抽屉 / 活动抽屉互斥（同一 `drawer` 槽位，主原型一次只开一个）。
- Code View 打开时不叠加 IO 抽屉为默认路径。

## 4. ImportDrawer

### 4.1 布局

```
┌─ 导入 ───────────────────────────── [×] ─┐
│ [ SQL ] [ DBML ] [ JSON ] [ 数据库 ]      │  ← format tabs（数据库 = 连接导入）
│ 数据库引擎: [ generic ▼ ]  （SQL 时显示）  │
│ ┌─────────────────────────────────────┐ │
│ │ 粘贴 SQL / 拖放 .sql 文件            │ │  ← import-textarea
│ └─────────────────────────────────────┘ │
│ 解析摘要: 3 条语句 · 预计 2 张表          │  ← import-parse-summary
│ [ 取消 ]              [ 导入到画布 ▶ ]   │
└──────────────────────────────────────────┘

数据库 Tab：
┌──────────────────────────────────────────┐
│ 引擎: [ SQLite ▼ ]                        │  ← import-db-engine（sqlite/postgres）
│ 连接: [ /path/to/db.sqlite 或 PG 连接串 ] │  ← import-db-source
│ Schema: [ public ]                       │  ← import-db-schema（仅引擎=postgres 时显示）
│ [ 连接并解析 ]                            │  ← import-db-connect
│ 解析摘要: 已连接 · 检测到 N 张表          │  ← 复用 import-parse-summary
│ [ 取消 ]              [ 导入到画布 ▶ ]   │
└──────────────────────────────────────────┘
```

- 宽度：**400px**（与 Inspector 默认 320px 区分，可挤压画布）
- testid：`import-drawer` / `import-format-tabs` / `import-engine-select` / `import-textarea` / `import-parse-summary` / `import-submit` / `import-cancel`
- 数据库来源新增 testid（feat-db-connect-import-and-ddl-realdb-verify）：`import-db-engine` / `import-db-source` / `import-db-connect`
- schema 输入 testid（fix-dbimport-save-and-pg-schema）：`import-db-schema` —— 仅在引擎选择 `postgres` 时渲染；占位与缺省值均为 `public`，留空按 `public` 发送；引擎为 `sqlite` 时不渲染且请求不携带 `schema`

### 4.2 格式 Tab

| format | 引擎选择 | 解析预览 |
|--------|----------|----------|
| `sql` | 必填（默认 `generic`） | `parse_sql_statements` 语句数 + 简单表名启发式（可选） |
| `dbml` | 隐藏 | 行数 / `Table` 块计数（纯函数 `count_dbml_tables`） |
| `json` | 隐藏 | `serde_json` 校验 + tables 数组长度 |
| `database` | 必填（`sqlite` / `postgres`，默认 `sqlite`） | 「连接并解析」→ `POST /api/v1/bridge/import/connect` 返回**结构化 tables JSON**（含表/列 comment）→ 复用 JSON 导入摘要口径（见 §4.4 第 3.5 条） |

### 4.3 文件拖放

- 接受扩展名：`.sql` / `.dbml` / `.json`
- 拖入后：自动切换对应 format Tab，内容写入 textarea
- 大小上限：读取 `bridge/config` 的 `maxImportSizeKb`（V1 可硬编码 5120 KB，与 bridge 默认一致）

### 4.4 提交行为

1. 校验：内容非空；SQL 需选引擎；JSON 需合法
2. **SQL 导入解析（relation-inspector-and-ddl-io：列级 DDL 解析）**——`parse_sql_import_tables` 口径：
   - 列定义：列名、类型整串原样保留（含 `VARCHAR(32)` / `DECIMAL(10,2)` 参数化）、列级约束 `PRIMARY KEY` / `NOT NULL` / `UNIQUE` / `AUTO_INCREMENT`（PG `SERIAL`/`BIGSERIAL` 类型 → `increment=true`）/ `DEFAULT <值>` / MySQL 行内 `COMMENT '...'`
   - **PG 风格 `COMMENT ON TABLE t IS '...'` 语句（fix-canvas-zoom-invite-comment-resize 新增）**：解析并回填对应表 `Table.comment`；**`COMMENT ON COLUMN t.c IS '...'`**：回填对应字段 `Field.comment`。目标表/列在本文本中不存在 → 跳过该语句（不报错，与 REFERENCES 悬空口径一致）。两种语句不计入「非 CREATE TABLE 语句」之外的错误路径
   - 表级约束：`PRIMARY KEY(a[, b])`（含联合主键）、`FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `c ... REFERENCES t(c2)` → 生成 `references` 连线（cardinality 按 core-01b §2 推导口径：目标列唯一/主键 → one_to_one，否则 one_to_many）
   - 标识符规范化：反引号 / 双引号 / 方括号去除；未知类型整串原样保留（不做方言映射）
   - 非 CREATE TABLE 语句跳过（**`COMMENT ON` 除外，见上**）；无合法 CREATE TABLE → 报错「未检测到数据表」
   - **V1 不做**：CHECK、INDEX/KEY 索引定义、ALTER TABLE 序列、视图/触发器、引擎方言全量映射
3. **本地合并进当前画布（fix-appbar-roomname-back-and-import-merge，替换原 bridge 新建游离 diagram + 跳转语义）**：
   - 编辑器上下文（房间编辑器 / 独立编辑器）下，解析结果经 `merge_import_into_store(store, tables, references)` 纯函数合并：新表**避让现有布局**排布（在现有内容右侧/下方网格落位，不覆盖既有表），references 一并追加
   - 合并后复用既有保存链路（`dirty` + `schedule_save` → PUT 当前 diagram），选中导入结果表并滚动可见
   - **不再**调用 `POST /api/v1/bridge/import/local`、**不再**创建游离 diagram、**不再** `window.location` 跳转；bridge 端点保留（契约不动），导入日志面板仅作历史记录展示
3.5. **数据库连接导入（feat-db-connect-import-and-ddl-realdb-verify；fix-canvas-zoom-invite-comment-resize 改结构化 JSON 直传）**——`database` Tab 提交流程：
   - 「连接并解析」：校验引擎与连接信息非空 → `POST /api/v1/bridge/import/connect`（`{ engine, source }`；引擎为 `postgres` 时携带 `schema`——输入框值去空白、为空则 `public`；`sqlite` 不携带，fix-dbimport-save-and-pg-schema）→ 200 取 `data.tables`（**结构化表数组**，含表/列 comment，契约见 `core-03-bridge-io.md` §13.1）→ 前端**复用 JSON 导入解析路径**（`parse_json_import_tables` 或对其的轻适配：把 `data.tables` 序列化为 JSON 导入同构文本后走同一纯函数）→ 解析摘要显示「已连接 · 检测到 N 张表」（N = `data.table_count`）；`table_count=0` → inline 错误「未检测到数据表」
   - **不再经 DDL 文本中转**：后端不再渲染 `ddl` 字符串，前端不再调用 `parse_sql_import_tables` 消费连接导入结果（SQL Tab 粘贴路径不变）
   - 端点错误文案：400 → 「不支持的数据库引擎」/「请填写连接信息」；502 → 「连接数据库失败，请检查连接信息」（inline 错误，不关抽屉）
   - 解析成功后「导入到画布」提交：与第 3 条**完全相同的本地合并管道**（`merge_import_into_store` 避让合并 → schedule_save PUT 落账 → Toast「已导入」→ 关闭抽屉），不调 `bridge/import/local`、不跳转页面
   - **注释保留**：导入产生的表/列 comment 原样进入 `Table.comment` / `Field.comment`，经 §3 合并与保存链路落库；展示/编辑入口见 `core-01a-table-and-field.md` §1.4
   - 连接信息（文件路径/连接串）仅用于当次请求，不写入 localStorage / 导入日志
   - 只读（Viewer）下 ImportDrawer 整体不可达（入口已禁用），数据库 Tab 不单独授权
3.6. **导入实体 ID 重键（fix-dbimport-save-and-pg-schema）**：
   - `merge_import_into_store` 在合并前对导入产生的新表/字段/关系统一做 ID 重键——以全局唯一 ID 生成器（`new_entity_id`，`{prefix}-{16位hex}`）替换解析期确定性 ID（`import-t-{i}` / `import-t-{i}-f{n}` / `import-r-{n}`），并按 旧→新 映射同步改写关系（reference）的 start/end table/field 四个端点
   - 原因：后端 `table`/`field`/`reference` 的 `id` 为全局单列主键，确定性 ID 在任意第二次导入保存时必然主键冲突（500）。重键后任意次数、任意 diagram 的导入保存均不冲突
   - 口径：重键只发生在合并入口一次；`parse_*_import_tables` 解析器内部 ID 形式不变（纯函数行为保持可测）
4. 成功 → Toast「已导入 N 张表 / M 条关系」+ 关闭抽屉
5. 失败（解析 Err / 保存失败）→ 抽屉内 inline 错误 + ErrorToast
6. 按钮文案：**导入到画布**；提交中 disabled + `导入中...`

> 兼容性说明：原第 3 条「`POST /api/v1/bridge/import/local`」与第 4 条「跳转 `/editor/{diagramId}`」废弃。原语义产生的游离 diagram 用户无法从房间列表触达，是「导入没有成功」反馈的根因。

### 4.5 入口汇总

| 入口 | testid | 行为 |
|------|--------|------|
| AppBar | `btn-import` | 打开 ImportDrawer |
| File 菜单 | `cdb-menu-import` | 打开 ImportDrawer（不再开 Import 模态） |
| EmptyGuide | `guide-import-sql` | 打开 ImportDrawer |
| ListView 工具条 | `list-btn-import` | 打开 ImportDrawer（本地合并管道，见 §4.4；只读禁用） |
| ListView 工具条 | `list-btn-export` | 打开 ExportDrawer（只读可用） |

> ListView 入口（fix-overflow-menu-room-delete-listview-io）：列表视图工具条右侧「导入」「导出」按钮打开与画布态相同的 IO 抽屉；导入提交走 §4.4 本地合并（不跳转、不调 bridge），合并后 ListView 表树刷新；交互细节见 `core-04` §10.5。

### 4.6 DDL 真实库验证口径

> 提案：feat-db-connect-import-and-ddl-realdb-verify

导出 DDL 的正确性验证从「文本断言」升级为「真实库执行」：

- 验证范围：本次承诺 **PostgreSQL + SQLite** 两路（MySQL/Oracle 等其余引擎仍文本断言，不在本变更）
- 验证方式：frontend-rs 集成测试（native target）调 `export_diagram_sql(store, engine)` 生成 DDL → 在**真实数据库实例**执行全部语句 → 断言执行成功且 `information_schema.tables` / `sqlite_master` 中建表数量、外键数量与模型一致
- **注释 round-trip（fix-canvas-zoom-invite-comment-resize 新增）**：PG 验证夹具中模型含表/列注释 → 执行导出 DDL 后直查 `obj_description()` / `col_description()` 与模型 `Table.comment` / `Field.comment` 一致；`COMMENT ON TABLE` 语句本身可被真实 PG 执行成功（语法验证）
- PG 实例：`postgresql_embedded`（dev-dependency，运行时从 Maven Central 拉取二进制，首次运行有下载耗时）
- SQLite 实例：sqlx-sqlite 临时文件（测试结束清理）
- 若生成 DDL 在真实库执行失败，视为导出器缺陷，须在本变更内修复生成逻辑（不接受跳过断言）

### 4.7 导入注释保留与引擎限制

> 提案：fix-canvas-zoom-invite-comment-resize

| 导入源 | 表注释 | 列注释 | 说明 |
|---|---|---|---|
| JSON（persistence 同构） | ✅ 透传 | ✅ 透传 | 既有能力，本提案补展示/编辑入口 |
| SQL 粘贴（MySQL 行内 `COMMENT '...'`） | ⚠️ 不支持 | ✅ 解析 | MySQL 行内表注释无标准语法位置（表选项 `COMMENT='...'` 解析器 V1 不解析），表注释丢弃属已知边界 |
| SQL 粘贴（PG `COMMENT ON TABLE/COLUMN`） | ✅ 解析（本提案新增） | ✅ 解析（本提案新增） | 支撑导出→导入 round-trip |
| 数据库连接（PostgreSQL） | ✅ `obj_description()` introspect | ✅ `col_description()` introspect | 结构化 JSON 直传 |
| 数据库连接（SQLite） | ❌ 引擎限制 | ❌ 引擎限制 | SQLite 无 COMMENT 语法；`comment` 字段返回空串，前端展示为空、不报错 |
| MySQL 数据库连接导入 | — | — | introspection 当前不支持 MySQL 引擎（`engine` 枚举仅 `sqlite`/`postgres`），不在本提案范围 |

- 引擎限制的口径：SQLite 导入结果中 comment 恒为空串，属**引擎能力缺失**而非缺陷；文档与 UI 均不承诺 SQLite 注释保留
- 数据库连接导入的注释完整性以 PostgreSQL 为准验收（见 §4.6 与测试用例 UT-PC-24）

## 5. ExportDrawer

### 5.1 布局

```
┌─ 导出 ───────────────────────────── [×] ─┐
│ [ SQL ] [ DBML ] [ JSON ]                 │
│ 数据库引擎: [ mysql ▼ ]  （SQL 时显示）    │
│ ┌─────────────────────────────────────┐ │
│ │ CREATE TABLE users ( ... );         │ │  ← export-preview（只读）
│ │ ...                                 │ │
│ └─────────────────────────────────────┘ │
│ [ 复制 ]  [ 下载 .sql ]                   │
└──────────────────────────────────────────┘
```

- testid：`export-drawer` / `export-format-tabs` / `export-engine-select` / `export-preview` / `export-copy` / `export-download`

### 5.2 预览生成（客户端）

| format | 函数 | V1 范围 |
|--------|------|---------|
| SQL | `export_diagram_sql(store, engine)` | `CREATE TABLE` + 列定义（类型整串 + PK/NOT NULL/**UNIQUE**/**DEFAULT**/**自增**（引擎映射：mysql/generic → `AUTO_INCREMENT`；postgresql 由 `SERIAL` 类型承担不追加）/**COMMENT**：mysql 行内 `COMMENT 'x'`，其余引擎后置 `COMMENT ON COLUMN t.c IS 'x'`）+ FK 引用 `references`（表末 `FOREIGN KEY (c) REFERENCES t(c2)`）+ **表注释（fix-canvas-zoom-invite-comment-resize 新增）**：mysql 表选项 `COMMENT='x'`（`CREATE TABLE ... ) COMMENT='x';`），postgresql/generic 后置 `COMMENT ON TABLE t IS 'x';`；表注释为空则不输出 |
| DBML | `export_diagram_dbml(store)` | 表 + 字段 + `ref:` 关系 |
| JSON | `serde_json::to_string_pretty(diagram)` | 与 persistence JSON 同构 |

空 diagram：预览区显示「暂无表，无法导出」；复制/下载 disabled。

- `COMMENT ON TABLE` 与 `COMMENT ON COLUMN` 按表分组输出在该表 `CREATE TABLE` 之后；单引号按既有列注释口径转义（`'` → `''`）
- 方言形态：

```sql
-- postgresql / generic
CREATE TABLE users ( ... );
COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.status IS '状态';

-- mysql（表选项，置于 CREATE TABLE 末尾）
CREATE TABLE users ( ... ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户表';
```

### 5.3 复制 / 下载

- **复制**：`navigator.clipboard.writeText`；成功按钮文案 `已复制` 2s
- **下载**：触发 `<a download>` blob；文件名 `{diagram_title}.{sql|dbml|json}`

### 5.4 导出到数据库（连接执行）

> 提案：feat-room-recycle-bin-dropdown-and-db-export

ExportDrawer SQL Tab 在「复制 / 下载」行下方新增「导出到数据库」区块（仅 SQL Tab 显示；DBML/JSON 不支持连接执行）：

```
┌─ 导出到数据库 ──────────────────────────┐
│ 引擎: [ sqlite ▼ ]                      │  ← export-db-engine（sqlite / postgres）
│ 连接信息: [_______________________]      │  ← export-db-source（SQLite 文件绝对路径 / PG 连接串）
│ [ 在数据库中执行 ]                       │  ← export-db-execute
│ 执行结果反馈区                           │  ← export-db-result
└──────────────────────────────────────────┘
```

- testid：`export-db-engine` / `export-db-source` / `export-db-execute` / `export-db-result`。
- 行为：
  - 点击「在数据库中执行」→ 取**当前预览区已生成的 SQL**（`export_diagram_sql` 输出）作为 `ddl`，调 `POST /api/v1/bridge/export/execute`（`{ engine, source, ddl }`，Bearer 鉴权）。
  - 空连接信息 → 内联错误「请填写连接信息」，不发请求；预览为空（空 diagram）→ 按钮 disabled。
  - 成功（200）→ `export-db-result` 显示「已执行 N 条语句 · 建表 M 张」+ Toast「已导出到数据库」。
  - 失败映射：400 →「不支持的数据库引擎」/「请填写连接信息」；502 →「导出执行失败：<引擎错误消息>」（直接展示后端 message，便于定位语法/连接问题）。
- 安全口径：连接信息仅当次请求使用，**不写入任何持久化存储**（localStorage / import log 均不涉及）；与 `core-03-bridge-io.md` §13 连接导入同一边界。
- 引擎下拉与连接执行不联动切换预览引擎：预览引擎（`export-engine-select`）与目标库引擎（`export-db-engine`）独立；用户须自行保证二者匹配（UI 文案提示「请确保导出引擎与目标库一致」）。

## 6. 与 V1 模态的关系

| V1 组件 | Phase C 处理 |
|---------|--------------|
| `ImportModal` | 保留代码供 UT-MM-10；File 菜单与 AppBar **不再**打开；可标记 `#[deprecated]` 注释 |
| `ImportSourceModal` | 不变；ImportDrawer 内嵌 `local` 源（不单独弹源选择） |
| Export 模态（未实现） | 由 ExportDrawer 完全替代 |

## 7. 样式

```css
.cdb-io-drawer {
  width: 400px;
  border-left: 1px solid var(--cdb-color-border);
  background: var(--cdb-color-bg);
  display: flex;
  flex-direction: column;
  z-index: 30; /* L3，与 Inspector 同级 */
}
.cdb-main.cdb-has-io-drawer {
  grid-template-columns: 48px 1fr 0 auto; /* 折叠 Inspector，显示 IO 抽屉 */
}
```

## 8. 测试 ID 索引

| TC ID | 描述 |
|-------|------|
| UT-PC-01 | `parse_sql_statements` 驱动导入摘要 |
| UT-PC-02 | `export_diagram_sql` 非空 diagram 输出含 `CREATE TABLE` |
| UT-PC-03 | `export_diagram_dbml` 含 `Table` 块 |
| UT-PC-04 | `IoDrawerKind` 互斥：开 Import 折叠 Inspector |
| UT-PC-05 | `count_dbml_tables` 纯函数 |
| ST-PC-01 | e2e：btn-import → 粘贴 SQL → 解析摘要可见 |
| UT-PC-11 | 后端 SQLite introspection → 方言 DDL（真实临时库） |
| UT-PC-12 | 后端 introspection DDL 渲染纯函数 + 错误映射（400/502/tables=0） |
| UT-PC-13 | 后端 PG introspection（嵌入式 PG） |
| UT-PC-14 | 导出 DDL 真实库执行验证（嵌入式 PG + SQLite） |
| UT-PC-15 | 前端锚点：ImportDrawer 数据库来源 testid 与接线 |
| ST-PC-04 | e2e：数据库来源连接导入 → 本地合并落账 |
| UT-PC-16 | 后端导出执行：SQLite 真实库执行 DDL（建表数/语句数） |
| UT-PC-17 | 后端导出执行：嵌入式 PG + 错误映射（400/502） |
| UT-PC-18 | 前端锚点：ExportDrawer 导出到数据库 testid 与接线 |
| UT-PC-19 | 样式锚点：`.cdb-form-select` / `.cdb-select` 视觉条款落地 |
| ST-PC-05 | e2e：导出到数据库 → mock 执行成功反馈 |

详细步骤见 `core-PC-import-export-test-cases.md`。

## 导入 / 导出格式统一约束

导入与导出均支持 **SQL / DBML / JSON** 三分段（`import-format-tabs` / `export-format-tabs`）：

| format | 导入 | 导出 |
|--------|------|------|
| SQL | 粘贴/拖入；引擎选择（生产）；解析摘要 | 客户端 SQL 预览 + 复制/下载 |
| DBML | 粘贴/拖入；表块计数 | DBML 预览 |
| JSON | 合法 JSON + tables | persistence 同构 JSON |

抽屉标题副文案对齐主原型：「SQL · DBML · JSON」。

## 9. Phase C 边界

- ❌ SQL/DBML 全屏代码视图（`btn-code-view`）— Phase D
- ❌ Mermaid / PNG 导出 — 后续提案
- ❌ 导入任务异步轮询 UI（logs/retry）— V1 仅同步成功路径 + 错误展示
- ✅ 右侧 IO 抽屉（Import + Export）
- ✅ bridge import API 接线
- ✅ 客户端 SQL/DBML/JSON 导出预览

---

### 9.1 统一原型边界补充

- ❌ 将主原型「模拟导入」标为已完成生产 bridge 接线
- ❌ 以 V1 Import 模态为默认用户路径（模态仅保留回归 UT）
- ✅ 更多菜单 → 导入/导出抽屉；SQL/DBML/JSON
- ✅ Viewer：导入提交与改写 diagram 的导出下载策略按房间角色；只读角色不得提交导入写库（与 `canEdit` 对齐，具体权限见 S04）

# Delta — core-01d-import-export.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## 2 组件树（E3 SideSheet 升级）

**merge 时在 §2 末尾追加**：

### §2.x E3 SideSheet 重构

V1 IO 抽屉用内嵌 `<aside class="cdb-io-drawer">` 自实现。E3 升级为 `<SideSheet placement=Right width=400>` 组件（来自 `core-09-core-components.md` §9）：

```rust
<SideSheet
    visible=io_drawer_open
    title=move || format!("{kind:?}")
    placement=SideSheetPlacement::Right
    width=400
    mask={true}
    mask_closable={true}
>
    <ImportExportContent kind=kind />
</SideSheet>
```

**Props 差异**（V1 → E3）：
- `cdb-is-io-drawer-open` class → `visible: RwSignal<bool>` prop
- `cdb-io-drawer__close` 内嵌按钮 → `<Button variant=Tertiary icon=IconClose />` in header
- 关闭动画：手动 CSS → E3 内置 `slide-in-right` / `slide-out-right`（E6 接入）

## 4–5 ImportDrawer / ExportDrawer（E2 复制/下载图标）

**merge 时在 §4、§5 各追加**：

### §4.x / §5.x 抽屉内操作按钮（E2 + E3）

| 行为 | 组件 | 视觉 |
|---|---|---|
| 复制导入源 / 复制导出结果 | `<Button variant=Secondary icon=IconCopy>复制</Button>` | E3 Button Secondary |
| 下载导出文件 | `<Button variant=Primary icon=IconDownload>下载</Button>` | E3 Button Primary |
| 拖入文件 | `<div class="cdb-dropzone"><IconUpload />"拖入文件或点击选择"</div>` | E2 Icon + E3 Collapse-style border |
| 切换数据库（MySQL/PostgreSQL/SQLite/...） | `<Dropdown trigger=Click position=BottomLeft>` | E3 Dropdown |

**ImportDrawer** 头部增加 `<Tag color=Info size=Small>SQL/DBML/JSON</Tag>` 标识当前 format。
