# 表与字段编辑规格（V1）

## 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`（`room-editor` 内表卡片 + `inspector`）。

| 项 | 约定 |
|---|---|
| 页面流 | 表/字段编辑发生在 `auth → rooms → room-editor` 之后的协作编辑器壳内 |
| 演示 ≠ 生产 | 主原型字段编辑为本地 state + 模拟保存；生产以 diagram REST / OT 为准 |
| 实现状态 | **后端已实现**；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| Inspector 锚点 | **`data-testid="inspector"`**（不是 `inspector-panel`） |

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
| 移动 | 拖拽表头（`data-drag-table`） | 拖动中未量化视觉坐标；关联关系线同帧跟随；`pointerup` 对齐网格后写入 undo |

**移动补充（对齐主原型 + optimize-canvas）**：

- **捕获**：`setPointerCapture`，避免跟丢。
- **rAF**：拖动中只更新表 `left/top` 与关系 `path[d]`；不得整页重渲染或整表 store set。
- **网格**：松手对齐；生产 **`GRID_SIZE = 20`**；主原型演示网格 12px，不作生产合同。
- **Viewer / 只读**：`canEdit() === false` 时 pointerdown 不得开始拖动；Inspector 输入 disabled。
- **锁定**：`locked === true` 时同 Viewer，不得开始拖动。

### 1.3 表级校验

- 表名非空
- 表名在当前 diagram 内唯一
- 表名长度 ≤ 64 字符
- 表名匹配 SQL 标识符规则（`^[A-Za-z_][A-Za-z0-9_]*$`）

### 1.4 表注释展示与编辑

> 提案：fix-canvas-zoom-invite-comment-resize

`Table.comment` 为 SQL 表注释（`COMMENT ON TABLE ...` / MySQL 表选项）。V1 仅有数据字段、无 UI 入口；本提案补齐展示与编辑。

#### 1.4.1 ListView 表树展示（只读）

- 表树节点（`list-tree-node-{表名}`，现状仅表名 + `N 字段` 计数）在表名下方追加注释行：
  - 有注释：`<span class="cdb-list-tree-node__comment" data-testid="list-table-comment-{table_id}">{comment}</span>`，单行截断（`text-overflow: ellipsis`），最大宽度与节点同宽
  - **无注释：不渲染该节点、不占位**，节点高度与现状一致
- 纯展示，点击行为不变（单击选中 / 双击跳转画布）

#### 1.4.2 Inspector 数据表 section 注释编辑（新增）

Inspector 顶部「数据表」section（`cdb-panel-section`，现有表名输入 `inspector-table-name` 与强调色）在表名输入下方追加注释输入：

```rust
<div class="cdb-form-group">
    <label>"注释"</label>
    <input
        class="cdb-form-input"
        data-testid="inspector-table-comment"
        prop:value=table_comment            // 受控：来自当前选中表 Table.comment
        disabled=ro                         // Viewer / locked 同 inspector-table-name
        on:blur=move |ev| {
            if !ro {
                on_set_table_comment(table_id, event_target_value(&ev));
            }
        }
    />
</div>
```

- **落账通路仿 `on_rename_table`**：blur 时若值变化 → 写 store（`Table.comment`）→ 进 `UndoRedoContext`（§4）→ `dirty` + `schedule_save` PUT 当前 diagram
- 表注释不参与 §1.3 校验（任意字符串，允许为空；不校验 SQL 标识符规则）
- 注释变更**不改表名**、不触发重名检查

#### 1.4.3 与导入/导出的关系

- 数据库导入（`core-01d` §4.4 第 3.5 条）与 JSON 导入透传的表/列注释落入同一 `Table.comment` / `Field.comment`，展示与编辑入口即时可见
- 导出（`core-01d` §5.2）在表注释非空时输出 `COMMENT ON TABLE`（PG/generic）或 MySQL 表选项 `COMMENT='...'`；空注释不输出

#### 1.4.4 testid 汇总

| testid | 元素 | 交互 |
|---|---|---|
| `list-table-comment-{table_id}` | 表树注释行 | 只读 |
| `inspector-table-comment` | Inspector 表注释输入 | blur 落账 |
| `inspector-field-comment-{field_id}` | Inspector 字段卡注释输入 | blur 落账 |
| `list-field-comment-{field_id}` | ListView 字段网格注释输入（已存在） | blur 落账 |

对应测试用例见 `core-PC-import-export-test-cases.md` UT-PC-29 / UT-PC-30 / ST-PC-08。

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
  dict_code: string;     // 绑定数据字典编码（S07，软引用；空串 = 未绑定）
}
```

> `dict_code` 为 S07（feat-data-dictionary）新增：软引用字典编码，空串表示未绑定；不参与 DDL 生成，仅文档语义。缺省/旧数据无此字段时视为空串（向前兼容）。绑定交互与级联口径详见 `core-01e-data-dictionary.md` §3。

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

### 2.3 字段操作

字段增删改、类型与约束的主编辑面为右侧 **Inspector**（选中表后展开），与主原型 `renderInspector` 一致：

- 表名 / 表注释（见 §1.4）/ 强调色
- 字段列表卡片：名称、类型、PK / NOT NULL / UNIQUE、**注释输入**（见下）、**数据字典绑定下拉**（S07 新增，`inspector-field-dict-{field_id}`，选项 = 当前图全部字典 + 「（不绑定）」；绑定后注释区下方显示映射摘要 Tag，口径见 `core-01e-data-dictionary.md` §3）、删除
- 「添加」字段、删除数据表（确认模态）

**字段注释编辑入口（双轨）**：

| 位置 | 形态 | testid | 说明 |
|---|---|---|---|
| ListView 字段网格第 2 列 | 行内注释输入（**已存在**，`editor_panels.rs` `list-field-comment-{fid}`） | `list-field-comment-{field_id}` | 受控输入，blur 落账；本提案不改动其行为 |
| Inspector 字段卡 | 每张字段卡片追加注释输入（**新增**） | `inspector-field-comment-{field_id}` | 受控输入，blur 落账通路仿 `on_rename`（写 store → undo → schedule_save） |

- 字段注释不参与 §2.4 校验（任意字符串，允许为空）
- Viewer（`canEdit() === false`）与 `locked === true` 时两处输入与字典绑定下拉均 disabled
- 字段注释与字典绑定变更进入 `UndoRedoContext`（§4）

历史左栏 Tables Tab 浏览能力已迁至 Command Palette / 画布选中；不以 V1 左栏 7 Tab 作为默认编辑路径。

### 2.4 字段级校验

- 字段名非空
- 字段名在当前 table 内唯一
- 字段名匹配 SQL 标识符规则
- 至少一个字段为 `primary`（V1 不强制）
- 自增字段必须是整数类型 + 主键
- ENUM 类型至少 1 个 values
- default 值需通过类型的 `checkDefault` 校验（drawdb `datatypes.js` 中的 `checkDefault` 函数）
- `dict_code` 不参与校验：任意已存在字典编码或空串均合法；指向已删除字典的悬空编码在加载时静默置空（见 `core-01e-data-dictionary.md` §2.4）

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

### 6.1 边界与对齐补充

- ❌ 要求生产端使用主原型 12px 网格步长
- ❌ Viewer 可拖表或改字段
- ✅ 表拖动 pointer 捕获 + rAF + 松手 `GRID_SIZE=20`
- ✅ Inspector `data-testid="inspector"` 为 e2e / 规格锚点

## 7. 对齐参考源

- drawdb `src/components/EditorCanvas/Table.jsx`
- drawdb `src/data/datatypes.js`（2,259 行）
- drawdb `src/utils/validateSchema.js`
- coldrawdb `frontend-rs/src/editor_panels.rs`（侧栏 TablesTab 引用）
- `database_design.json` 样本

---
# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E2 增量）

## 2 字段（Field）— 字段类型徽章 E2 Tag + Icon

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

## 2 主键/外键图标（E2 Icon 替换）

**merge 时在 §2 主键/外键说明处替换**：

- 主键：`<IconKey />` + `<Tag color=Warning size=Small>PK</Tag>`（`--cdb-color-warning` 文字色）
- 外键：`<IconLink />` + `<Tag color=Info size=Small>FK</Tag>`
- 索引：`<IconIndex />` + `<Tag color=Neutral size=Small>IDX</Tag>`
- 唯一约束：`<IconUnique />` + `<Tag color=Success size=Small>UQ</Tag>`
- 非空：`<IconNotNull />`（无 Tag，纯图标 hover 提示）
