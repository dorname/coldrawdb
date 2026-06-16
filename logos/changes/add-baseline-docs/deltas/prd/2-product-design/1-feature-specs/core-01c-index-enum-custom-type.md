## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md`
> 策略：移除文件开头的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` 标记块及其紧随的 `>` 元数据行，保留正文首个一级标题以下所有内容原样。

# 索引 / 枚举 / 自定义类型规格（V1）

> **V1 关键边界**：本章三个对象（Index / Enum / CustomType）在 coldrawdb V1 中**仅前端 state**；后端 11 张表未独立建表承载，导出 SQL 时由前端组合生成。
> drawdb 主分支的 `indice / indice_link / enum / type` 四张表对应在 coldrawdb V1 的后端不实体化。

## 1. 索引（Index）

### 1.1 索引对象结构

```ts
interface Index {
  id: string;
  name: string;             // 索引名
  unique: boolean;          // UNIQUE 索引
  fields: string[];         // 字段 id 列表（顺序敏感）
  type: "BTREE" | "HASH" | "FULLTEXT" | "SPATIAL" | "";  // 索引类型（部分引擎支持子集）
}
```

### 1.2 索引操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 表内"Index"按钮或快捷键 `I` | 默认空 Index，name="idx_N" |
| 重命名 | 行编辑 | `name` 变更 |
| 切唯一 | 复选框 | `unique` 变更 |
| 加字段 | 字段多选下拉 | `fields` 追加 |
| 删字段 | 字段标签 × | `fields` 移除 |
| 改类型 | 类型下拉 | `type` 变更（按引擎过滤可用选项） |
| 删除 | 行右键 | 从 table 中移除 |

### 1.3 索引级校验

- 索引名非空
- 索引名在当前 table 内唯一
- 至少 1 个字段
- FULLTEXT / SPATIAL 字段类型兼容性：仅特定类型（如 TEXT）可建
- 引擎支持子集：FULLTEXT 在 MySQL/MariaDB；SPATIAL 在 MySQL/MariaDB/PostgreSQL；HASH 在 PostgreSQL/MSSQL

### 1.4 索引 ↔ 后端对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Index` | `indice` + `indice_link` | drawdb 后端有实体；coldrawdb V1 仅前端 state，导出 SQL 时**不**生成 CREATE INDEX 语句（V1 行为） |
| `unique` | `indice.unique` | 仅 drawdb 持久化；V1 状态 |
| `type` | `indice.type` | 仅 drawdb 持久化；V1 状态 |

> V1 SQL 导出**不**包含索引 DDL；用户需自行在数据库中创建。**V2 计划**：导出包含 CREATE INDEX。

## 2. 枚举（Enum）

### 2.1 枚举对象结构

```ts
interface Enum {
  id: string;
  name: string;             // 枚举名
  values: string[];         // 枚举值列表（至少 1 个）
}
```

### 2.2 枚举操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 侧栏 Enums Tab "+" | 默认 name="enum_N"，values=[] |
| 重命名 | 行编辑 | `name` 变更 |
| 加值 | 输入框回车 | `values` 追加 |
| 改值 | 单元格编辑 | `values[i]` 变更 |
| 删值 | 行右键 | `values` 移除 |
| 删除枚举 | 列表项右键 | 从 diagram 中移除 |

### 2.3 枚举级校验

- 枚举名非空
- 枚举名在当前 diagram 内唯一
- 至少 1 个 values 项
- values 项非空字符串
- 引用一致性：若某字段 `type = '<enum_name>'`，删除该枚举前需用户确认级联影响

### 2.4 枚举 ↔ 后端对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Enum` | （coldrawdb V1 **无独立表**） | drawdb 主分支有 `enum` 表；coldrawdb V1 仅前端 state，导出 SQL 时按引擎生成 `ENUM(...)` 或 `CREATE TYPE ... AS ENUM` |
| `values` | 同上 | 同上 |

> V1 引擎适配：MySQL/MariaDB 字段内嵌 `ENUM('v1','v2')`；PostgreSQL 单独 `CREATE TYPE ... AS ENUM ('v1','v2')`；其他引擎降级为 `VARCHAR` + `CHECK`。

## 3. 自定义类型（CustomType）

### 3.1 自定义类型对象结构

```ts
interface CustomType {
  id: string;
  name: string;                  // 类型名
  equivalent: string;            // 等价基础类型（如 INT / VARCHAR）
  fields: {                      // 复合类型子字段（仅 equivalent 为复合类型时）
    name: string;
    type: string;
  }[];
}
```

### 3.2 自定义类型操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 顶部菜单"Configure Custom Types" | 打开 `ConfigureCustomTypes` 模态 |
| 重命名 | 行编辑 | `name` 变更 |
| 改等价基础类型 | 引擎类型下拉 | `equivalent` 变更 |
| 加子字段 | 子字段列表"+" | `fields` 追加 |
| 改子字段名/类型 | 行编辑 | `fields[i]` 变更 |
| 删子字段 | 行右键 | `fields` 移除 |
| 删除 | 列表项右键 | 从 diagram 中移除 |

### 3.3 自定义类型级校验

- 类型名非空
- 类型名在当前 diagram 内唯一
- `equivalent` 必须是 7 引擎中某一引擎支持的基础类型
- 复合类型（`fields.length > 0`）仅当 `equivalent` 为复合类型（如 OBJECT）时允许
- 引用一致性：某字段 `type = '<custom_type_name>'` 时删除需用户确认

### 3.4 自定义类型 ↔ 后端对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `CustomType` | （coldrawdb V1 **无独立表**） | drawdb 主分支有 `type` 表；coldrawdb V1 仅前端 state，导出 SQL 时按引擎生成 `CREATE TYPE ... AS OBJECT (...)`（OracleSQL）或忽略（其他引擎） |

> V1 自定义类型仅 OracleSQL 引擎会展开为 `CREATE TYPE`；其他引擎导出为 `equivalent` 基础类型。

## 4. V1 存储边界总结

| 对象 | 画布展示 | 前端 state | 后端独立表 | 导出 SQL |
|---|---|---|---|---|
| Index | ✅ 表内嵌 | ✅ | ❌（drawdb 有 indice/indice_link） | ❌（V1 不生成） |
| Enum | ✅ 侧栏 | ✅ | ❌（drawdb 有 enum） | ✅ 按引擎生成 |
| CustomType | ✅ 模态管理 | ✅ | ❌（drawdb 有 type） | ⚠️ 仅 OracleSQL 展开 |

> 此边界是 coldrawdb V1 的有意简化。完整持久化在 V2 计划中（需新增 `indice/indice_link/enum/type` 表）。

## 5. 撤销 / 重做

三者的所有操作（创建 / 编辑 / 删除）都进入 `UndoRedoContext`。V1 不持久化。

## 6. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-IE-01 | 表加 FULLTEXT 索引 → 导出 MySQL SQL 含 `FULLTEXT KEY` |
| UT-IE-02 | 删索引 → SQL 不再含该索引 |
| UT-IE-03 | 枚举加 3 个值 → MySQL 字段导出 `ENUM('a','b','c')` |
| UT-IE-04 | 枚举加 3 个值 → PostgreSQL 导出 `CREATE TYPE ... AS ENUM` |
| UT-IE-05 | 自定义类型 equivalent=VARCHAR → SQLite 导出降级为 VARCHAR |
| UT-IE-06 | 自定义类型 equivalent=OBJECT → OracleSQL 导出 `CREATE TYPE ... AS OBJECT` |
| ST-IE-01 | 端到端：表+索引+枚举+自定义类型 → 全引擎导出 SQL 正确 |

## 7. V1 边界

- ❌ 索引 DDL 写入导出 SQL（V1 不生成 CREATE INDEX）
- ❌ 枚举 / 自定义类型后端持久化（V1 仅前端 state）
- ❌ 自定义类型在多引擎一致性（V1 仅 OracleSQL 完整展开）
- ❌ 跨 diagram 共享枚举 / 类型（V1 单 diagram 内）

## 8. 对齐参考源

- drawdb `src/components/EditorSidePanel/EnumsTab/`
- drawdb `src/components/EditorSidePanel/TypesTab/`
- drawdb `src/components/Modals/ConfigureCustomTypes/`
- drawdb `src/data/datatypes.js`（引擎 → 类型映射）
- drawdb `src/utils/exportSQL/...`（SQL 导出对账）
- coldrawdb `frontend-rs/src/editor_panels.rs`（侧栏 EnumsTab 引用）
- `docs/drawdb-capability-checklist.md` §1.4 / §1.7 / §1.8

---
# Delta — core-01c-index-enum-custom-type.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## MODIFIED — §1 索引（Index）— E3 Tag 视觉

**merge 时在 §1 末尾追加**：

### §1.x 索引徽章（E3 Tag）

| 索引类型 | E3 Tag color | 标签文本 | Icon |
|---|---|---|---|
| INDEX（普通） | `Neutral` | "IDX" | `<IconIndex />` |
| UNIQUE | `Success` | "UQ" | `<IconUnique />` |
| PRIMARY | `Warning` | "PK" | `<IconKey />` |
| FULLTEXT | `Info` | "FT" | `<IconString />` |
| SPATIAL | `Info` | "SP" | `<IconLink />` |

**视觉**：inline-flex 20px 高，gap 4px，Tag 不带 icon 文字（仅文字）或带 icon 文字混合

## MODIFIED — §2 枚举（Enum）— E3 Collapse

**merge 时在 §2 末尾追加**：

### §2.x 枚举折叠面板（E3 Collapse）

V1 枚举详情用模态（`cdb-modal-enum`）。E3 升级为 E3 `<Collapse>` 内嵌在 Inspector 抽屉内：

```rust
<Collapse lazy_render={true} bordered={CollapseBordered::Default}>
  <CollapsePanel header=view! { <Tag color=Primary>ENUM</Tag> <span>{name}</span> }>
    <For each=values key=|v| v.id children=|v| view! {
      <div class="cdb-py-2">
        <Tag color=Neutral>{v.name}</Tag>
      </div>
    } />
  </CollapsePanel>
</Collapse>
```

## MODIFIED — §3 自定义类型（CustomType）— E3 Tag

**merge 时在 §3 末尾追加**：

### §3.x 自定义类型徽章（E3 Tag）

| 类型 | E3 Tag color | Icon |
|---|---|---|
| 域（domain） | `Primary` | `<IconType />` |
| 复合类型（composite） | `Info` | `<IconString />` |
| 范围（range） | `Neutral` | `<IconInt />` |

