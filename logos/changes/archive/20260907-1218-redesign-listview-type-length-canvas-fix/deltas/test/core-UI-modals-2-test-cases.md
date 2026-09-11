## MODIFIED — UT-MM-36 按引擎字段类型清单纯函数测试

附录 A 中 UT-MM-36 行更新为：

| UT-MM-36 | 按引擎字段类型清单纯函数测试（types_for_database 返回**基类型**清单：Generic 6 项含 UUID/VARCHAR；MySQL 含 DATETIME/DECIMAL 无 SERIAL；PostgreSQL 含 UUID/BOOLEAN/SERIAL/TIMESTAMP/NUMERIC 无 DATETIME；Sqlite/Mssql/Oracle 回落 Generic；清单均为裸基类型，不含 `VARCHAR(255)` 等参数化整串） | `editor_core.rs::types::types_for_database` |

## MODIFIED — ST-LV-01 ListView 按表分组 e2e

附录 A 中 ST-LV-01 行更新为：

| ST-LV-01 | ListView 按表分组编辑网格 e2e（两表各出组头行「表名 (N 字段)」+ 字段行；点组头选中表 Inspector 同步；字段行内嵌编辑控件；分组固定按表，原分组切换/批量类型面板由行内编辑取代） | `frontend-rs/scripts/test-spec-parity-d.mjs` |

## MODIFIED — ST-DB-02 引擎创建后锁定 + PG 清单内容

正文与附录断言口径更新（其余步骤不变）：类型下拉清单改断言**基类型**——`UUID/INT/BIGINT/SERIAL/VARCHAR/TEXT/BOOLEAN/TIMESTAMP/NUMERIC`（含 UUID、BOOLEAN、SERIAL，无 DATETIME，无 `VARCHAR(255)`/`NUMERIC(10,2)` 参数化整串项）；当前类型基类型 UUID 在清单内无占位重复。附录 A 中 ST-DB-02 行更新为：

| ST-DB-02 | 引擎创建后锁定 + PG 基类型清单（AppBar 下拉禁用；PG 含 UUID/BOOLEAN/SERIAL 无 DATETIME、清单为裸基类型；当前类型无占位重复） | `frontend-rs/scripts/test-spec-parity-d.mjs` |

## ADDED — UT-MM-37 参数化类型解析/合成纯函数测试

### UT-MM-37 — parse_field_type / compose_field_type 纯函数测试

- **位置**：`frontend-rs/src/editor_core.rs`（`parse_field_type` / `compose_field_type`）
- **步骤与预期**：
  1. parse `"VARCHAR(32)"` → base=VARCHAR, len="32", scale=""；parse `"NUMERIC(10,2)"` → base=NUMERIC, len="10", scale="2"；parse `"UUID"` → base=UUID, len/scale 空；parse 非法串（小写/无括号匹配）→ 整体视为 base（占位兼容）
  2. compose(VARCHAR, "", "") → `"VARCHAR(255)"`（缺省长度）；compose(VARCHAR, "32", "") → `"VARCHAR(32)"`；compose(NUMERIC, "", "") → `"NUMERIC(10,2)"`；compose(DECIMAL, "12", "4") → `"DECIMAL(12,4)"`；compose(UUID, _, _) → `"UUID"`
  3. 往返一致：compose(parse(`"VARCHAR(32)"`)) == `"VARCHAR(32)"`；老整串 `"VARCHAR(255)"`/`"DECIMAL(10,2)"` 解析后基类型正确（向后兼容）

附录 A 新增行：

| UT-MM-37 | 参数化类型解析/合成纯函数测试（parse/compose：VARCHAR 长度、DECIMAL/NUMERIC 精度小数位、缺省值、老整串兼容、往返一致） | `editor_core.rs::parse_field_type` + `compose_field_type` |
