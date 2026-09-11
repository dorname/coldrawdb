## MODIFIED — 2.2 7 引擎类型映射（CAP-DATATYPES-*）

### 2.2 7 引擎类型映射（CAP-DATATYPES-*）

详细类型清单见 `src/data/datatypes.js`（2,259 行）。下表给出每引擎的预置类型集合：

| 引擎 | 类型集合（节选） | 特殊能力 |
|---|---|---|
| MySQL | INT / BIGINT / VARCHAR / TEXT / DATE / DATETIME / TIMESTAMP / DECIMAL / FLOAT / DOUBLE / BLOB / JSON / BOOLEAN / ENUM | hasUnsignedTypes |
| PostgreSQL | + JSONB / UUID / SERIAL / ENUM / ARRAY | hasTypes / hasEnums / hasArrays |
| SQLite | INT / INTEGER / TEXT / REAL / BLOB / NUMERIC | 弱类型（5 种） |
| MariaDB | 同 MySQL + BOOLEAN | hasUnsignedTypes |
| MSSQL | INT / BIGINT / VARCHAR / NVARCHAR / TEXT / NTEXT / DATETIME / DATETIME2 / BIT / DECIMAL / FLOAT / REAL | （无 unsigned） |
| OracleSQL | + VARCHAR2 / NUMBER / CLOB / BLOB / DATE / TIMESTAMP | （无 unsigned） |
| Generic | INT / VARCHAR / TEXT / DATE / BOOLEAN | 通用基线 |

**编辑器 MVP 类型清单（基类型）**（Inspector / ListView 类型下拉的唯一权威口径；生产 `editor_core.rs::types::types_for_database` 与原型 `typesForDatabase` 必须与此严格一致）：

| 引擎 | 基类型清单（有序） |
|---|---|
| Generic | UUID / INT / BIGINT / VARCHAR / TEXT / BOOLEAN |
| MySQL | INT / BIGINT / VARCHAR / TEXT / DATETIME / DECIMAL |
| PostgreSQL | UUID / INT / BIGINT / SERIAL / VARCHAR / TEXT / BOOLEAN / TIMESTAMP / NUMERIC |

**参数化类型口径**（redesign-listview-type-length-canvas-fix 新增，PDManer 式「类型 + 长度 + 小数位」分离配置）：

- `Field.type_` 落账为**合成整串**：VARCHAR → `VARCHAR(长度)`；DECIMAL / NUMERIC → `DECIMAL(精度,小数位)` / `NUMERIC(精度,小数位)`；其余基类型落账裸串（如 `UUID`、`TIMESTAMP`）
- 默认参数：VARCHAR 长度缺省 **255**；DECIMAL/NUMERIC 精度缺省 **10**、小数位缺省 **2**
- 解析/合成由纯函数承担（生产 `parse_field_type` / `compose_field_type`，原型 `parseFieldType` / `composeFieldType`）：
  - parse：`"VARCHAR(32)"` → `{base:"VARCHAR", len:"32", scale:""}`；`"NUMERIC(10,2)"` → `{base:"NUMERIC", len:"10", scale:"2"}`；裸串 `"UUID"` → `{base:"UUID", len:"", scale:""}`；无法解析时整体视为 base（占位兼容）
  - compose：VARCHAR 系缺省补 `(255)`；DECIMAL/NUMERIC 系缺省补 `(10,2)`；其余原样返回 base
- **向后兼容**：老数据整串（`VARCHAR(255)`、`DECIMAL(10,2)`）均可被 parse 正确拆解；长度/小数位列仅对 VARCHAR 系 / DECIMAL·NUMERIC 系可编辑，其余禁用
- 占位追加规则不变：当前类型的**基类型**不在清单时，下拉追加该基类型占位项保留显示（不改写既有字段）

约束：

- 短清单是上表预置集合的子集，面向 MVP 常用类型；Sqlite/Mssql/Oracle 回落 Generic 组
- 导出（SQL/DBML）直接使用 `type_` 合成整串，无需感知参数化
