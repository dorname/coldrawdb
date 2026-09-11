# 实现任务：diagram-database-dialect

> 配套：`logos/changes/diagram-database-dialect/proposal.md`
> 上游：用户反馈问题 2（2026-09-04）→ p0-fix「不在范围」转出
> 外环强制约束：禁止改测试断言；新 UT/ST 编号先 grep 取下一空闲（UT-MM 占用至 35；ST 新增 ST-DB-01）；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）
> 勘察结论：Database 枚举 + store.database + 序列化通路已存在（editor_core.rs:125 / editor_data_access.rs:1470），本提案只接 UI 与联动

## [code] 类型清单纯函数

- [x] `frontend-rs/src/editor_core.rs`: `pub fn types_for_database(db: Database) -> &'static [&'static str]`：
  - Generic：INT / BIGINT / VARCHAR(255) / TEXT / BOOLEAN（对齐既有硬编码 5 项，回归不破）
  - Mysql：INT / BIGINT / VARCHAR(255) / TEXT / DATETIME / DECIMAL(10,2)
  - Postgresql：INT / BIGINT / VARCHAR(255) / TEXT / SERIAL / TIMESTAMP / NUMERIC(10,2)
- [x] UT-MM-36：`types_for_database` 三组清单内容 + Generic 与既有 5 项一致

## [code] AppBar 引擎切换

- [x] `frontend-rs/src/editor_panels.rs`: AppBar 加引擎下拉（data-testid=`app-db-select`，三项 Generic/MySQL/PostgreSQL，当前值 = store.database）
- [x] 切换 → `store.database.set` + dirty + schedule_save（复用既有保存通路）

## [code] Inspector 类型过滤 + 导出跟随

- [x] `frontend-rs/src/editor_panels.rs`: `inspector-field-type` 选项改由 `types_for_database(store.database.get())` 渲染（当前类型不在清单时追加一个 selected 占位项，避免显示丢失）
- [x] `frontend-rs/src/editor_panels.rs`: ExportDrawer `engine` 信号初始值改读 `store.database`（Generic→generic / Mysql→mysql / Postgresql→postgresql，其余映射 generic）
- [x] `frontend-rs/src/editor_panels.rs`: CodeView SQL 由硬编码 "generic" 改读 `store.database`

## 测试

- [x] UT-MM-36 → `core-UI-modals-2-test-cases.md` 登记 + reporter `UT_PASS_IDS` 落账
- [x] ST-DB-01 → `test-spec-parity-d.mjs`：AppBar 切 MySQL → PUT 落账 database=mysql → Inspector 类型下拉含 DATETIME 不含 SERIAL → ExportDrawer 默认 mysql → 切回 Generic 回归
- [x] ST-DB-01 → `core-UI-modals-2-test-cases.md` 登记 + jsonl 落账

## 不在范围

- Sqlite/Mssql/Oracle UI 暴露；类型转换迁移；引擎专属 DDL 深度定制
- 导出引擎手改不回写图（单向跟随）
- 不修改既有测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

> 偏差留痕：Inspector `inspector-field-type`（SelectionKind::Field 面板）在生产 UI 无构造入口（死代码），实际活路径为 Inspector 表面板字段行 `type-{fid}` 下拉——两处均已接 `types_for_database`；e2e 断言走活路径。侧栏 TablesTab 字段类型下拉（含 UUID/DATE 8 项）未过滤，留作后续评估。
