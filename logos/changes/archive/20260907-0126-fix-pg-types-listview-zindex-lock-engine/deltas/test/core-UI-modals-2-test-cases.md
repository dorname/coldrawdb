## REMOVED — ST-DB-01 图级引擎切换 e2e

从附录 A 用例清单删除 ST-DB-01 行。

**删除原因**：S04.1 新增「引擎创建后锁定」约束（fix-pg-types-listview-zindex-lock-engine）——diagram 引擎在创建房间时确定，进入房间编辑器后 AppBar `app-db-select` 禁用。ST-DB-01 的前提「AppBar 切引擎 → PUT 落账」语义作废，由 ST-DB-02 的锁定语义替代；类型清单按引擎过滤的断言并入 ST-DB-02。

## ADDED — ST-DB-02 引擎创建后锁定 + PostgreSQL 类型清单

### ST-DB-02 — 引擎创建后锁定 + PG 清单内容（fix-pg-types-listview-zindex-lock-engine）

- 前置：已登录；房间 diagram 引擎 = PostgreSQL（创建房间时选定）；画布含一张带 UUID 主键字段的表
- 步骤：
  1. 进入房间编辑器，查看 AppBar `[data-testid="app-db-select"]`
  2. 选中该表，查看 Inspector 字段类型下拉 `[data-testid="inspector-field-type"]`
- 预期：
  1. `app-db-select` **disabled**（只读标识）且显示 PostgreSQL；不产生任何切换请求
  2. 类型下拉清单 = `UUID/INT/BIGINT/SERIAL/VARCHAR(255)/TEXT/BOOLEAN/TIMESTAMP/NUMERIC(10,2)`：含 UUID、BOOLEAN、SERIAL，**无 DATETIME**
  3. 当前类型 UUID 已在清单内，**无占位追加**——下拉中 UUID 只出现一次（回归：修复前 PG 清单缺 UUID，兜底占位逻辑导致重复 UUID）
- 对齐实现：`frontend-rs/src/editor_panels.rs`（AppBar 锁定 + Inspector 清单）；`frontend-rs/src/editor_core.rs::types::types_for_database`

附录 A 新增行：

| ST-DB-02 | 引擎创建后锁定 + PG 清单内容（AppBar 下拉禁用；PG 含 UUID/BOOLEAN/SERIAL 无 DATETIME；当前类型无占位重复） | `frontend-rs/scripts/test-spec-parity-d.mjs` |

## MODIFIED — UT-MM-36 按引擎字段类型清单纯函数测试

附录 A 中 UT-MM-36 行更新为：

| UT-MM-36 | 按引擎字段类型清单纯函数测试（types_for_database：Generic 6 项含 UUID 回归不破；MySQL 含 DATETIME/DECIMAL(10,2) 无 SERIAL 不变；PostgreSQL 含 UUID/BOOLEAN/SERIAL/TIMESTAMP/NUMERIC(10,2) 无 DATETIME；Sqlite/Mssql/Oracle 回落 Generic） | `editor_core.rs::types::types_for_database` |
