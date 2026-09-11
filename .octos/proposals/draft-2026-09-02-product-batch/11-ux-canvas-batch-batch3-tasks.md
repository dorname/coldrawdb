# 实现任务 — ux-canvas-batch 批次 3（细化 tasks v2，C-1 通用决策程序 + C-3 schema 导出 + P3 触发链）

> 状态：**细化 tasks v2**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`logos/changes/ux-canvas-batch/proposal.md` + `tasks.md`（commit `be8da4d`）
> 上游裁决链：黑板条目6 operator 批注（Q1=全部 9 项列表视图候选）→ 外环判词 C-1..C-4 → 批次 1..2 全采认 → 条目 9 修复 ACK → 外环条目 11 打回修订 v2（4 处语义层修正）
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 取下一空闲（**UT-MM-26 起**——grep 确认 UT-MM-25 已被 ux-canvas-batch 批次 2 收尾占用）
> **C-3 裁决（外环代决，R3 技术取舍）**：导出**降格为仅 CSV**，纯手写无依赖；不引入 xlsx（zip 二进制 + wasm 体积代价，YAGNI）

## v1 → v2 修订点速查（外环条目 11 反馈）

| v1 问题 | v2 修正 |
|---|---|
| **P1** 实例推演练 5 自相矛盾未修正（:41）：文本写「name→DECIMAL(改)」又自注「不兼容，应跳过」 | 场景 5 期望改为 **name 跳过**（VARCHAR→DECIMAL = 字符串→数值 = 不兼容）；删自注 |
| **P2** 类型兼容性缺通用决策规则（C-1 未闭环）：真值表只列 10 行示例对，未覆盖的对无规则可依 | **v2 必须给出确定性决策程序**：①解析基类型 + 可选 `(n)` 参数；②同基类型族内窄→宽直接改/宽→窄跳过；③族间一律跳过；④未列出的类型对保守 fallback 跳过；⑤各族给出至少一个收窄反向实例推演 |
| **P3** 批量改类型无 UI 触发链（条目 9 同类断链风险的预防）：只有 `batch_change_types` 纯函数，无 UI 入口、无模态、无触发链 | **v2 必须补**：ListView 批量改类型 UI（字段多选 + 目标类型输入 + 触发按钮 testid `list-view-batch-type`）+ 模态（`ModalKind::BatchType`）+ 触发链全链（按钮→modal_kind 置位→AppRoot modals 渲染→Apply 调 `batch_change_types`→写 store 走 CommandStack/OT 通路→`store.dirty.set(true)`） |
| **P4** CSV 导出内容语义错误：实例推演把导出内容写成**数据行**（`id=1, name=apple` → `1,apple`）——本工具是 schema 设计器，没有数据行 | **v2 必须重写**：导出内容定义 = 列表视图本身的 schema 内容（行=字段,列=table_name/field_name/field_type/has_index,与批次 1 展示列对齐）；表头行（`table_name,field_name,field_type,has_index`）+ 全部实例推演按 schema 内容重写（转义真值表仍然适用——表名/字段名可含逗号引号换行）；`export_tables_csv` 的输入签名随之修正为 `&[Table]` |

## [code] 代码实现（批次 3：批量改类型 + 双击跳画布 + 导出仅 CSV schema 内容）

### 批量改类型（多字段一次性改类型）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `batch_change_types` 纯函数（UT-MM-26）：
  - 输入：`tables: &mut Vec<Table>`、`field_type_map: HashMap<String, String>`（字段 ID → 新类型）
  - **v2 类型兼容性通用决策程序（C-1 闭环）**——按下列确定性规则执行，不留白：

    **步骤 ①：解析基类型 + 可选 `(n)` 参数**
    - 输入类型字符串格式：`INT`、`BIGINT`、`DECIMAL(p,s)`、`VARCHAR(n)`、`BOOLEAN`、`DATE`、`DATETIME`
    - 解析函数签名：`fn parse_type(s: &str) -> (BaseType, Option<u32>, Option<u32>)`（DECIMAL 有两个参数，其他有一个或零个）

    **步骤 ②：定义类型族白名单**
    - **数值族**：`SMALLINT < INT < BIGINT < DECIMAL(p,s) < FLOAT < DOUBLE`（**由窄到宽白名单**，序关系为兼容性判定基线）
    - **字符串族**：`CHAR(n) < VARCHAR(n) < TEXT < LONGTEXT`
    - **日期族**：`DATE < DATETIME < TIMESTAMP`
    - **布尔族**：仅 `BOOLEAN`（自成一族，无窄宽）
    - **二进制族**：`BLOB < MEDIUMBLOB < LONGBLOB`
    - 注：族与族之间**无兼容关系**

    **步骤 ③：族内由窄到宽 → 直接改；族内由宽到窄 → 跳过（精度/数值收窄需用户确认，不静默执行）**
    - 解析后比较族内位置：源位置 < 目标位置 → 由窄到宽 → 直接改
    - 源位置 > 目标位置 → 由宽到窄 → 跳过
    - 源位置 = 目标位置 → 同型 → 直接改
    - **同基类型参数收窄**（如 VARCHAR(255)→VARCHAR(50)）→ 跳过（精度收窄需用户确认）

    **步骤 ④：跨族一律跳过**
    - 数值族→字符串族、字符串族→数值族、数值族→日期族等任何跨族 → 跳过

    **步骤 ⑤：未列出的类型对保守 fallback = 跳过**
    - 解析失败、空字符串、不在族白名单的基类型 → 跳过
    - **不抛错、不静默执行降级**——C-1 边界强制

    **步骤 ⑥：非法/空目标类型跳过**
    - 目标类型为空字符串、解析失败、不在白名单 → 跳过

    **各族收窄反向实例推演（C-1 强制）**：
    - **数值族收窄反向**：`BIGINT → INT`（宽→窄） → 跳过 / `DECIMAL(10,2) → DECIMAL(5,0)`（精度收窄） → 跳过 / `FLOAT → INT`（宽→窄） → 跳过
    - **字符串族收窄反向**：`VARCHAR(255) → VARCHAR(50)`（参数收窄） → 跳过 / `TEXT → VARCHAR(100)`（族内收窄） → 跳过
    - **日期族收窄反向**：`DATETIME → DATE`（精度收窄） → 跳过 / `TIMESTAMP → DATETIME`（精度收窄） → 跳过
    - **跨族**：`INT → VARCHAR`（数值→字符串） → 跳过 / `VARCHAR → INT`（字符串→数值） → 跳过 / `DATE → DATETIME`（族内由窄到宽） → 直接改

  - **真值表**（v2 修正版——含各族收窄反向 + 跨族 + 非法目标类型）：

    | 源类型 | 目标类型 | 决策程序路径 | 结果 |
    |---|---|---|---|
    | INT | BIGINT | 数值族由窄到宽（步骤 ③） | 直接改 |
    | BIGINT | INT | 数值族由宽到窄（步骤 ③） | **跳过**（C-1 收窄反向） |
    | INT | VARCHAR | 跨族（步骤 ④） | **跳过**（VARCHAR→INT 修正为数值族→字符串族跨族） |
    | VARCHAR | INT | 跨族（步骤 ④） | **跳过** |
    | VARCHAR(255) | VARCHAR(50) | 字符串族同型参数收窄（步骤 ③ 同型后参数变化） | **跳过**（精度收窄 C-1 边界） |
    | BOOLEAN | INT | 跨族（步骤 ④） | **跳过** |
    | DATE | DATETIME | 日期族由窄到宽（步骤 ③） | 直接改 |
    | DATETIME | DATE | 日期族由宽到窄（步骤 ③） | **跳过** |
    | (任意) | "" | 非法目标类型（步骤 ⑥） | **跳过** |
    | (任意) | "INVALID_TYPE" | 解析失败/不在白名单（步骤 ⑤） | **跳过** |

  - **v2 实例推演**（外环条目 11 P1 修正——v1 演练 5 自相矛盾已删）：
    - 场景 1：fields = [id(INT), name(VARCHAR(255))], field_type_map = {id→BIGINT, name→INT} → id→BIGINT（数值族由窄到宽改）、name→**跳过**（字符串族→数值族跨族步骤 ④）
    - 场景 2：fields = [id(INT), name(VARCHAR(255))], field_type_map = {id→INT, name→VARCHAR(50)} → id→INT（同型改）、name→**跳过**（字符串族参数收窄步骤 ③）
    - 场景 3：fields = [id(INT)], field_type_map = {id→"INVALID_TYPE"} → id→**跳过**（解析失败步骤 ⑤）
    - 场景 4：fields = [id(INT)], field_type_map = {id→VARCHAR} → id→**跳过**（INT→VARCHAR = 数值族→字符串族跨族步骤 ④）
    - 场景 5（v2 修正）：fields = [id(BOOLEAN), name(VARCHAR(255))], field_type_map = {id→INT, name→DECIMAL} → id→**跳过**（BOOLEAN→INT 跨族步骤 ④）、name→**跳过**（字符串族→数值族跨族步骤 ④）

### 批量改类型 UI 触发链（P3 强制——条目 9 同类断链风险的预防；外环条目 12 修正 4：统一为 checkbox 多选 + 单一目标类型，删 modal-input-batch-type 手输框）

- [ ] `frontend-rs/src/editor_panels.rs`: ListView 组件内**批量改类型 UI**（**外环条目 12 修正 4——checkbox 多选 + 单一目标类型**）：
  - 字段多选（checkbox）：data-testid `list-view-select-field-{field_id}`（每行字段一行）
  - 目标类型输入（text input 或下拉）：data-testid `list-view-batch-type-target`
  - 触发按钮：data-testid `list-view-batch-type`，on:click 调 `modal_kind.set(Some(modals::ModalKind::BatchType))`（范式参照批次 2 `list-view-batch-rename` 改派四步）
- [ ] `frontend-rs/src/editor_panels.rs`: `ModalKind` 新增 `BatchType` 变体（外环条目 11 P3 强制——不沿用现有模态避免越界）
- [ ] `frontend-rs/src/editor_panels.rs`: `BatchTypeModal` 组件（**外环条目 12 修正 4——展示已选字段清单（按字段名）+ 确认目标类型（只读回显 `list-view-batch-type-target` 的值），删 `modal-input-batch-type` 手输框**）：
  - 模态容器：data-testid `modal-batch-type`
  - 已选字段清单（按字段名展示，**只读**）：data-testid `modal-batch-type-selected-fields`
  - 目标类型确认（**只读回显**，值来自 ListView 的 `list-view-batch-type-target`）：data-testid `modal-batch-type-target-display`
  - Apply 按钮：data-testid `modal-submit-batch-type`，on:click 调 `batch_change_types` → 写 store 走 CommandStack/OT 通路 → `store.dirty.set(true)`
  - **`field_type_map` 由「checkbox 选中集 × 目标类型」在 Apply 时构造**（外环条目 12 修正 4 强制——字段 ID 是内部标识用户不可见，手输框与 checkbox 二源矛盾，统一为 checkbox + 单一目标类型）
- [ ] `frontend-rs/src/editor_panels.rs`: AppRoot modals 渲染 match 加 `Some(ModalKind::BatchType) => view! { ... }` 分支（范式参照 `:8117` BatchRename）
- [ ] **触发链全链核验清单**（P3 强制）：
  - 按钮 on:click（`list-view-batch-type`）→ `modal_kind.set(Some(modals::ModalKind::BatchType))`
  - AppRoot modals match 渲染 `BatchTypeModal`
  - BatchTypeModal Apply 调 `batch_change_types` → `store.tables.update` + `store.dirty.set(true)`

### 双击跳画布对应表

- [ ] `frontend-rs/src/editor_panels.rs`: ListView 组件内双击行（`on:dblclick`）调 `on_jump_to_canvas(Some(table_id.clone()))`
  - ListView 加 `on_jump_to_canvas: Rc<dyn Fn(Option<String>)>` prop（外环条目 10 强制）
  - AppRoot 调用点传 `on_jump_to_canvas=move |id: Option<String>| { view_mode.set(ViewMode::Canvas); if let Some(tid) = id { on_select_table(Some(tid)); } }`（**同时切回 ViewMode::Canvas + 选中态走既有 on_select_table 通路**——外环条目 10 强制）
  - data-testid 命名：`list-view-dblclick-{table_id}`（沿用既有规范；保留 on:click + 加 on:dblclick 共存，**禁止改既有 data-testid**）

### 导出仅 CSV schema 内容（P4 强制——v1 错误内容修正）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `export_tables_csv` 纯函数（UT-MM-27）：
  - **v2 签名修正**：`pub fn export_tables_csv(tables: &[Table]) -> String`（输入是 `&[Table]` 而非字段值行——本工具是 schema 设计器，无数据行）
  - **v2 导出内容定义** = 列表视图本身的 schema 内容（行 = 字段，列 = `table_name,field_name,field_type,has_index`）：
    - 表头行：`table_name,field_name,field_type,has_index`
    - 数据行：每个 `Table.fields` 的每个 `Field` 一行
      - 列 1：`Table.name`
      - 列 2：`Field.name`
      - 列 3：`Field.type_`
      - 列 4：`yes` 或 `no`（按 `Table.indices.is_empty()` 反向：空 = `no`，非空 = `yes`）
  - **不引入 xlsx 依赖**（YAGNI，wasm 体积代价）——纯手写 CSV 序列化
  - **v2 CSV 转义真值表**（与 v1 一致，转义真值表仍然适用——表名/字段名可含逗号引号换行）：

    | 字符 | 在字段值中 | CSV 输出 | 实例推演 |
    |---|---|---|---|
    | `,`（逗号）| 是 | 字段加双引号 `"..."` | `table_name=users,posts` → `"users,posts"` |
    | `"`（双引号）| 是 | 字段加双引号 + 内部双引号转义为 `""` | `name=she said "hi"` → `"she said ""hi"""` |
    | `\n`（换行）| 是 | 字段加双引号 `"..."` | `name=line1\nline2` → `"line1\nline2"` |
    | 三者均无 | 否 | 字段不加引号 | `name=users` → `users` |
    | 字段为空 | — | 空字段 | `name=,` → `name=,` |

  - **v2 实例推演**（P4 重写——按 schema 内容）：
    - 表 [users(id INT pk), name VARCHAR(255)] → `users,id,INT,yes\nusers,name,VARCHAR(255),yes`
    - 表 [users(id INT), posts(id INT)] → `users,id,INT,no\nposts,id,INT,no`
    - 表 [bad,name="weird,name"] → `bad,name="weird,name",VARCHAR(255),no`（逗号转义）
    - 表 [bad,name=she said "hi"] → `bad,name="she said ""hi""",VARCHAR(255),no`（引号转义）
    - 表 [bad,name="line1\nline2"] → `bad,name="line1\nline2",VARCHAR(255),no`（换行转义）
    - 空表 `[]` → `table_name,field_name,field_type,has_index\n`（仅表头）
- [ ] `frontend-rs/src/editor_panels.rs`: 导出 CSV UI（ListView 组件内）：
  - 导出 CSV 按钮：data-testid `list-view-export-csv`
  - 弹出下载（前端用 `Blob` + `URL.createObjectURL` + `<a download>` 触发下载）：data-testid `list-view-export-csv-download`

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-26，**v2 更新——覆盖通用决策程序各族收窄反向 + 跨族 + 非法目标类型**）：
  - happy: `batch_change_types(tables, {field_id→BIGINT})` → BIGINT（数值族由窄到宽步骤 ③ 直接改）
  - happy: `batch_change_types(tables, {field_id→INT})` → INT（同型直接改）
  - happy: `batch_change_types(tables, {field_id→VARCHAR})` → **跳过**（跨族步骤 ④；v1 测试覆盖错误，v2 修正）
  - happy: `batch_change_types(tables, {field_id→VARCHAR(50)})` → 跳过（字符串族参数收窄步骤 ③）
  - edge: `batch_change_types(tables, {field_id→"INVALID_TYPE"})` → 跳过（解析失败步骤 ⑤）
  - edge: `batch_change_types(tables, {field_id→""})` → 跳过（非法目标类型步骤 ⑥）
  - edge: `batch_change_types(tables, {field_id→DATETIME})` → DATE→DATETIME（日期族由窄到宽步骤 ③ 直接改）
  - edge: `batch_change_types(tables, {})` → 全部不变（空 field_type_map）
  - **v2 新增** edge: `batch_change_types(tables, {field_id→SMALLINT})` → INT→SMALLINT（数值族由宽到窄步骤 ③ **跳过**）
  - **v2 新增** edge: `batch_change_types(tables, {field_id→DATE})` → DATETIME→DATE（日期族由宽到窄步骤 ③ **跳过**）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-27，**v2 修正——输入为 `&[Table]` 按 schema 内容导出**）：
  - happy: `export_tables_csv([Table{name=users, fields=[Field{name=id, type_=INT, ...}]}])` → `table_name,field_name,field_type,has_index\nusers,id,INT,yes`（**v2 重写——按 schema 内容，非数据行**）
  - happy: `export_tables_csv([Table{name=users, fields=[Field{name=id, type_=INT}], indices=[]}])` → `...users,id,INT,no`
  - happy: `export_tables_csv([Table{name=users, fields=[Field{name=posts, type_=VARCHAR(255)}]}])` → `...users,posts,VARCHAR(255),no`
  - happy: `export_tables_csv([Table{name=bad, fields=[Field{name='she said "hi"', type_=VARCHAR(255)}]}])` → `...bad,"she said ""hi""",VARCHAR(255),no`（引号转义）
  - edge: `export_tables_csv([])` → `table_name,field_name,field_type,has_index\n`（空表——仅表头）
  - edge: `export_tables_csv([Table{name="line1\nline2", fields=[Field{name=id, type_=INT}]}])` → `..."line1\nline2",id,INT,no`（换行转义）
  - edge: `export_tables_csv([Table{name="weird,name", fields=[Field{name=id, type_=INT}]}])` → `..."weird,name",id,INT,no`（表名含逗号——转义）

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-26/27 行：
  ```
  | UT-MM-26 | 列表视图批量改类型纯函数测试（类型兼容性通用决策程序：族内由窄到宽直接改/由宽到窄跳过/跨族跳过/未列出对保守 fallback 跳过/非法目标类型跳过） | `editor_panels.rs::batch_change_types` |
  | UT-MM-27 | 列表视图导出 CSV schema 内容纯函数测试（CSV 转义：逗号/引号/换行；输入 `&[Table]`，行=字段，列=table_name,field_name,field_type,has_index——C-3 裁决仅 CSV 纯手写无依赖） | `editor_panels.rs::export_tables_csv` |
  ```
- [ ] 确认 UT-MM-26/27 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议（批次 3）

1. `batch_change_types` 纯函数（v2 通用决策程序 + 真值表 + 实例推演）+ UT-MM-26（10 子用例：v2 新增各族收窄反向）
2. 批量改类型 UI（ListView 加字段多选 + 目标类型输入 + 触发按钮 + `ModalKind::BatchType` + `BatchTypeModal` + AppRoot modals 渲染分支 + 触发链全链核验）
3. 双击跳画布对应表（ListView 加 `on_jump_to_canvas` prop + `on:dblclick` + AppRoot 调用点切回 Canvas + 选中态走既有 `on_select_table` 通路）
4. `export_tables_csv` 纯函数（v2 输入 `&[Table]` + schema 内容导出 + CSV 转义真值表 + v2 实例推演）+ UT-MM-27（7 子用例：v2 重写按 schema 内容）
5. 导出 CSV UI（ListView 组件内：导出按钮 + 下载触发）
6. spec 登记（UT-MM-26/27 行）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 批量重命名（多表或多字段一次性改名）——批次 2 已实现
- 过滤（按名称模糊匹配/按类型/按是否有索引）——批次 2 已实现
- 导出 xlsx（zip 二进制 + wasm 体积代价）——**C-3 裁决降格为仅 CSV**，本批不做
- 列宽可调——批次 4
- 表/字段分组（按 schema/按 tag）——批次 4
- 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + 关键交互帧率 < 16ms（**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**）+ rAF 统一调度）——批次 4
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 不修改 `Reference`/`Table` struct 数据契约
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

## 外环判词强制约束落实（v2）

- **C-1**：批量改类型类型兼容性通用决策程序已闭环（步骤 ①~⑥ + 各族收窄反向实例推演 + 真实值表）
- **C-3 裁决**：导出仅 CSV 纯手写无依赖；不引入 xlsx；CSV 转义真值表仍然适用（表名/字段名可含逗号引号换行）
- **P1**：实例推演练 5 已修正为 name 跳过（删自注）
- **P2**：类型兼容性通用决策程序 6 步骤完整给出（解析/族内窄宽/跨族跳过/未列出保守 fallback/各族收窄反向实例推演）
- **P3**：批量改类型 UI 触发链全链已补（按钮 testid + ModalKind 置位 + AppRoot 渲染 + Apply 写 store CommandStack/OT + dirty）
- **P4**：CSV 导出内容已重写为 schema 内容（行=字段，列=table_name/field_name/field_type/has_index；签名改 `&[Table]`；实例推演按 schema 内容重写）
- **新 UT 编号**：UT-MM-26/27（grep 确认 UT-MM-25 已被 ux-canvas-batch 批次 2 收尾占用，UT-MM-26 起空闲）