# 表与字段编辑规格（V1）

## 1. 表（Table）

### 1.1 表对象结构

```ts
interface Table {
  id: string;            // 客户端 UUID（与后端一致）
  name: string;          // 表名（必填，唯一）
  x: number;             // 画布 x 坐标
  y: number;             // 画布 y 坐标
  locked: boolean;       // 锁定（防止误操作）
  comment: string;       // 表注释（SQL COMMENT）
  color: string;         // 表头颜色（hex，如 "#175e7a"）
  fields: Field[];       // 字段列表
  indices: Index[];      // 索引列表（详见 core-01c）
  width: number;         // 渲染宽度（auto 时为 0）
}
```

### 1.2 表操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 侧栏"+"或快捷键 `T` | 分配 UUID + 默认坐标 |
| 重命名 | 双击表名 | `name` 变更 |
| 移动 | 拖拽标题栏 | `{x, y}` 变更 |
| 缩放 | 拖拽右下角 | `width` 变更（高度自适应） |
| 锁定 | 右键菜单 / 工具栏 | `locked = true` 后禁止移动/编辑 |
| 复制 | Ctrl/Cmd + D | 深拷贝 + 偏移 (20, 20) |
| 删除 | Delete / 右键 | 从 diagram 中移除；级联删除字段 |
| 改色 | ColorPicker | `color` 变更 |

### 1.3 表级校验

- 表名非空
- 表名在当前 diagram 内唯一
- 表名长度 ≤ 64 字符
- 表名匹配 SQL 标识符规则（`^[A-Za-z_][A-Za-z0-9_]*$`）

## 2. 字段（Field）

### 2.1 字段对象结构

```ts
interface Field {
  id: string;
  name: string;          // 字段名（必填）
  type: string;          // 字段类型（CAP-DATATYPES-*）
  size: number | "";     // 大小（VARCHAR(255) 等）
  default: string;       // 默认值
  check: string;         // CHECK 表达式
  primary: boolean;      // 主键
  unique: boolean;       // 唯一
  notNull: boolean;      // 非空
  increment: boolean;    // 自增（AUTO_INCREMENT / SERIAL）
  comment: string;       // 字段注释
  values: string[];      // ENUM 值列表（仅 ENUM 类型）
}
```

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

### 2.3 字段操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 添加 | 表内底部"+"或快捷键 | 默认字段：name="field_N" type="VARCHAR" size=255 |
| 重命名 | 双击 | `name` 变更 |
| 改类型 | 类型下拉 | `type` 变更 + 联动 `size` 默认值 |
| 改大小 | size 输入框 | `size` 变更 |
| 改默认值 | default 输入框 | `default` 变更（含类型校验） |
| 改 check | check 输入框 | `check` 变更 |
| 切主键/唯一/非空/自增 | 复选框 | 标志位变更 |
| 改注释 | comment 输入框 | `comment` 变更 |
| 排序 | 拖拽字段行 | 字段顺序变更（影响 SQL 生成） |
| 删除 | 行右键菜单 | 从 table 中移除 |

### 2.4 字段级校验

- 字段名非空
- 字段名在当前 table 内唯一
- 字段名匹配 SQL 标识符规则
- 至少一个字段为 `primary`（V1 不强制）
- 自增字段必须是整数类型 + 主键
- ENUM 类型至少 1 个 values
- default 值需通过类型的 `checkDefault` 校验（drawdb `datatypes.js` 中的 `checkDefault` 函数）

## 3. 与后端实体的对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Table` | `table` + `field` + `table_link` + `indice` + `indice_link` | 表的字段、索引通过关联表连接 |
| `Field` | `field` | 字段独立行 |
| `color` | （coldrawdb V1 **未独立存储**，drawdb 有） | drawdb 支持表头颜色；coldrawdb V1 仅前端 state |
| `locked` | （coldrawdb V1 **未独立存储**） | drawdb 支持锁定；coldrawdb V1 仅前端 state |
| `width` | （coldrawdb V1 **未独立存储**） | 同上 |

> `color / locked / width` 在 coldrawdb V1 中仅前端状态；后端不持久化（详见 §6 边界）。

## 4. 撤销 / 重做

字段级别的所有变更（添加 / 重命名 / 改类型 / 删除 / 排序）都进入 `UndoRedoContext`。V1 不支持跨进程持久化。

## 5. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-T-01 | 创建表 → 字段默认填充 |
| UT-T-02 | 重命名表 → 校验重名 |
| UT-T-03 | 添加字段 → 默认值 + check |
| UT-T-04 | 改字段类型为 ENUM → 校验 values |
| UT-T-05 | 字段排序拖拽 → SQL 输出顺序 |
| ST-T-01 | 端到端：创建 5 张表 20 字段 → 保存 → 重新加载 → 一致 |

## 6. V1 边界

- ❌ 表头颜色持久化（V1 仅前端）
- ❌ 表锁定状态持久化（V1 仅前端）
- ❌ 字段级权限控制（V1 不做）

## 7. 对齐参考源

- drawdb `src/components/EditorCanvas/Table.jsx`
- drawdb `src/data/datatypes.js`（2,259 行）
- drawdb `src/utils/validateSchema.js`
- coldrawdb `frontend-rs/src/editor_panels.rs`（侧栏 TablesTab 引用）
- `database_design.json` 样本

---
# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E2 增量）

## MODIFIED — §2 字段（Field）— 字段类型徽章 E2 Tag + Icon

**merge 时在 §2 末尾追加**：

### §2.x 字段类型徽章（E2 Tag + Icon）

V1 字段类型用 `text-orange-500`（main `stringColor`）等 Tailwind 颜色 + emoji 表示。E2 升级为 E3 `<Tag color=... size=Small>` + E2 Icon：

| 字段类型 | E2 图标 | E3 Tag color | 颜色 token |
|---|---|---|---|
| string / varchar / text | `<IconString />` | `Primary` | `--cdb-color-primary-soft` |
| int / integer / bigint | `<IconInt />` | `Warning` | `--cdb-color-warning-soft` |
| decimal / numeric / float | `<IconDecimal />` | `Success` | `--cdb-color-success-soft` |
| boolean / bool | `<IconBoolean />` | `Info` | `--cdb-color-info-soft` |
| date / datetime / timestamp | `<IconDate />` | `Info` | `--cdb-color-info-soft` |
| enum (drawdb enum) | `<IconEnum />` | `Primary` | `--cdb-color-primary-soft` |
| binary / blob | `<IconBinary />` | `Success` | `--cdb-color-success-soft` |

**Props 签名**：
```rust
<FieldTypeBadge type_: FieldType, size: TagSize = TagSize::Small />
```

**视觉**：inline-flex 22px 高，gap 4px，Icon size=12，Tag 不带文字（仅 icon + soft 背景）

## MODIFIED — §2 主键/外键图标（E2 Icon 替换）

**merge 时在 §2 主键/外键说明处替换**：

- 主键：`<IconKey />` + `<Tag color=Warning size=Small>PK</Tag>`（`--cdb-color-warning` 文字色）
- 外键：`<IconLink />` + `<Tag color=Info size=Small>FK</Tag>`
- 索引：`<IconIndex />` + `<Tag color=Neutral size=Small>IDX</Tag>`
- 唯一约束：`<IconUnique />` + `<Tag color=Success size=Small>UQ</Tag>`
- 非空：`<IconNotNull />`（无 Tag，纯图标 hover 提示）

