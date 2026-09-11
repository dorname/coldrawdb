# Delta — core-01a-table-and-field.md（字段扩展 dict_code 绑定属性）

## MODIFIED — 2.1 字段对象结构

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
  notNull: boolean;     // 非空
  increment: boolean;    // 自增（AUTO_INCREMENT / SERIAL）
  comment: string;       // 字段注释
  values: string[];      // ENUM 值列表（仅 ENUM 类型）
  dict_code: string;     // 绑定数据字典编码（S07，软引用；空串 = 未绑定）
}
```

> `dict_code` 为 S07（feat-data-dictionary）新增：软引用字典编码，空串表示未绑定；不参与 DDL 生成，仅文档语义。缺省/旧数据无此字段时视为空串（向前兼容）。绑定交互与级联口径详见 `core-01e-data-dictionary.md` §3。

## MODIFIED — 2.3 字段操作

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

## MODIFIED — 2.4 字段级校验

- 字段名非空
- 字段名在当前 table 内唯一
- 字段名匹配 SQL 标识符规则
- 至少一个字段为 `primary`（V1 不强制）
- 自增字段必须是整数类型 + 主键
- ENUM 类型至少 1 个 values
- default 值需通过类型的 `checkDefault` 校验（drawdb `datatypes.js` 中的 `checkDefault` 函数）
- `dict_code` 不参与校验：任意已存在字典编码或空串均合法；指向已删除字典的悬空编码在加载时静默置空（见 `core-01e-data-dictionary.md` §2.4）
