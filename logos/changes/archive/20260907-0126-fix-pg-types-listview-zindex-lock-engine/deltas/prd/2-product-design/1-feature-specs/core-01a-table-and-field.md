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

**编辑器 MVP 类型下拉短清单**（Inspector 字段类型下拉的唯一权威口径；生产 `editor_core.rs::types::types_for_database` 与原型 `typesForDatabase` 必须与此严格一致）：

| 引擎 | 下拉短清单（有序） |
|---|---|
| Generic | UUID / INT / BIGINT / VARCHAR(255) / TEXT / BOOLEAN |
| MySQL | INT / BIGINT / VARCHAR(255) / TEXT / DATETIME / DECIMAL(10,2) |
| PostgreSQL | UUID / INT / BIGINT / SERIAL / VARCHAR(255) / TEXT / BOOLEAN / TIMESTAMP / NUMERIC(10,2) |

约束：

- 短清单是上表预置集合的子集，面向 MVP 常用类型；Sqlite/Mssql/Oracle 回落 Generic 组
- 字段当前类型不在短清单时，Inspector 追加占位项保留显示（不改写既有字段）；短清单覆盖后该兜底不应触发（修复前 PG 缺 UUID 导致 UUID 字段出现重复占位项）
