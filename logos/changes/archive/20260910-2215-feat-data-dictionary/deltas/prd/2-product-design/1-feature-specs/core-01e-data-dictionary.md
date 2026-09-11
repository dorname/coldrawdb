# Delta — core-01e-data-dictionary.md（新增文件）

> 目标主文档：`logos/resources/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md`（新文件，merge 时整体创建）

## ADDED — 全文（新文件）

# 数据字典规格（S07 / CAP-DICT-*）

> **定位**：PDManer 式「代码映射表管理」——集中定义字段可枚举取值及含义（`0=否/1=是`、`1=男/2=女`），供各表字段绑定复用，避免重复维护同义映射。
> **存储口径**：与 Enum/CustomType 一致（core-01c §0）——**仅前端 state，随 diagram JSON 快照持久化**；后端不实体化字典表，导出由前端组合生成。
> **与 Enum 的分工**：Enum 决定字段存储类型（`ENUM(...)` / `CREATE TYPE`）；数据字典描述字段取值的**业务语义映射**，不改变 DDL 类型，二者可同时存在。

## 1. 数据模型

```ts
interface DataDictionary {
  id: string;            // 内部 id
  name: string;          // 字典名称（必填，如「是否」「性别」）
  code: string;          // 字典编码（必填，同图内唯一，如 yes_no / gender）
  comment: string;       // 字典说明（可空）
  items: DictionaryItem[];
}

interface DictionaryItem {
  id: string;
  value: string;         // 映射值（必填，字典内唯一，如 "1"）
  label: string;         // 含义（必填，如 "男"）
  sort: number;          // 展示顺序
}

// Field 扩展（core-01a §2.1 追加）：
//   dict_code: string    // 绑定字典编码（软引用，空串 = 未绑定）
```

**diagram JSON 持久化**：顶层新增可选 `dictionaries: DataDictionary[]`；缺省/缺字段 = 空数组（旧图向前兼容）。保存/加载沿用 S01 PUT 快照与 `?share=` 只读通路，无新端点。

## 2. 字典 Tab（侧栏）

入口：侧栏新增「字典」Tab（与 Enums/Types 并列，core-04 体系）。

### 2.1 列表项

每项：字典名称 + 编码 + 字典项数量 + **引用计数**（绑定该字典的字段数）。

### 2.2 操作

| 操作 | 触发 | 效果 | testid |
|---|---|---|---|
| 单击 | 鼠标左键 | 展开字典详情（字典项列表，见 §2.3） | `dict-item-{id}` |
| 双击 | 鼠标左键 | 重命名字典名称 | — |
| 右键 | 鼠标右键 | 上下文菜单（重命名 / 编辑编码 / 删除） | — |
| "+" | Tab 底部 | 新建字典（默认 `dict_N` / `code_N`） | `dict-add` |
| 导出 | Tab 工具条 | 导出数据字典 Markdown（§4） | `dict-export` |

### 2.3 字典项编辑（详情面板）

- 表格行：`value | label` 两列行内编辑（blur 落账）+ 行尾删除按钮；底部「+ 添加项」`dict-item-add-{dict_id}`
- 上移/下移调整 `sort`（作用于选中行）
- 校验（实时，行内红标）：value 非空且字典内唯一；label 非空

### 2.4 校验与引用一致性

- 字典名/编码非空；编码同图内唯一（冲突时红标 + 阻断落账）
- **删除被引用字典**：引用计数 > 0 时弹确认「被 N 个字段引用，删除后这些字段的绑定将置空，是否继续？」；确认后所有 `dict_code = 该编码` 的字段置空（同一次 undo 事务）
- 删除未被引用字典：无需确认，可撤销

## 3. 字段绑定（Inspector）

- Inspector 字段卡新增「数据字典」下拉：选项 = 当前图全部字典（显示 `名称（编码）`）+ 顶部「（不绑定）」；testid `inspector-field-dict-{field_id}`
- 绑定为**软引用**（存字典编码）：字典改名不影响绑定；改编码时同步改写所有引用字段的 `dict_code`（同一次 undo 事务）
- 绑定展示：字段卡的注释区下方以 Tag 显示字典映射摘要（如 `1=男 2=女`，超 4 项折叠为 `共 N 项`）；ListView 字段网格「说明」列后缀追加同样摘要（只读，不自 ListView 编辑绑定）
- 绑定/解绑即改即账，进入 UndoRedoContext，沿用 S01 debounce 保存通路
- Viewer / locked：下拉 disabled；导出可用

## 4. Markdown 导出（CAP-DICT-04）

「导出」生成单个 `.md` 文件，三段结构：

```markdown
# 数据字典 — {diagram_name}

## 字典清单
| 字典名称 | 编码 | 项数 | 引用字段数 | 说明 |

## 字典明细
### {name}（{code}）
| 值 | 含义 |

## 字段绑定关系
| 表 | 字段 | 字段注释 | 绑定字典 |
```

- 空字典（无 items）仍列于清单与明细（明细表头下空表）
- 无字典时「导出」禁用（title 说明「当前图表没有数据字典」）
- 下载走浏览器 Blob 下载，文件名 `data-dictionary-{diagram_name}.md`

## 5. 协作与只读

- S05 OT：字典/字典项/绑定编辑均为模型 op，随既有 op 通路广播与合并，**协议零变更**；远端 op 到达后字典 Tab 与 Inspector 摘要即时刷新
- S02 分享只读：字典 Tab 可浏览、可导出，编辑控件禁用
- S06 MCP：本变更不扩展 MCP 工具；`dictionaries` 随 diagram JSON 读写自然透出

## 6. 撤销 / 重做

字典 CRUD、字典项编辑、绑定/解绑、引用删除级联置空均进 UndoRedoContext；级联置空与字典删除为**单次 undo 单元**。

## 7. V1 边界（本变更不做）

- ❌ 字典驱动的代码生成（Java Bean / MyBatisPlus 枚举常量）
- ❌ Word / Excel 导出（首阶段仅 Markdown）
- ❌ 跨图表 / 全局字典库（作用域 = 单图表）
- ❌ 字典值对字段取值的运行时校验（绑定仅文档语义）
- ❌ 字典影响 DDL 生成（不改 `type_`，不生成 CHECK 约束）

## 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-S07-01 | 新建字典 → 列表出现，默认名/编码 |
| UT-S07-02 | 编码重复 → 红标阻断落账 |
| UT-S07-03 | 字典项 value 重复 → 行内红标 |
| UT-S07-04 | 绑定字段 → 字段卡显示映射摘要 Tag |
| UT-S07-05 | 改字典编码 → 引用字段 dict_code 同步改写 |
| UT-S07-06 | 删除被引用字典 → 确认后字段绑定置空（单次 undo 可恢复） |
| UT-S07-07 | Markdown 导出内容 = 当前模型（清单/明细/绑定三段） |
| UT-S07-08 | 无字典时导出禁用 |
| ST-S07-01 | 端到端：建字典 → 绑字段 → 保存 → 重开 → 绑定仍在 |
| ST-S07-02 | 旧图（无 dictionaries 字段）加载 → 默认为空不报错 |
| ST-S07-03 | Viewer 只读：编辑禁用、导出可用 |

## 9. 对齐参考源

- PDManer 数据字典（字典管理 / 字段绑定 / 导出）
- coldrawdb `core-01c-index-enum-custom-type.md` §2（Enum 前端 state 口径）
- coldrawdb `frontend-rs/src/editor_panels.rs`（EnumsTab 参照实现）
