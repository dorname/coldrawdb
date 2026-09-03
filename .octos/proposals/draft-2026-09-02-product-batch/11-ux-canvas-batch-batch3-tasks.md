# 实现任务 — ux-canvas-batch 批次 3（细化 tasks，C-1 + C-3 落实）

> 状态：**细化 tasks**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`logos/changes/ux-canvas-batch/proposal.md` + `tasks.md`（commit `be8da4d`）
> 上游裁决链：黑板条目6 operator 批注（Q1=全部 9 项列表视图候选）→ 外环判词 C-1..C-4 + 记一笔 → 批次 1 复验采认（commit `039c93d`）→ 批次 2 细化 tasks + B2-S1 补充规则 → 批次 2 复验采认（commit `1d21a4e`）→ 批次 2 UI 收尾（commit `22bc68a` + `c507bb5`）→ 条目 9 修复 ACK（commit `be0cd48`）→ 外环条目 10 派发批次 3 细化 tasks
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 取下一空闲（**UT-MM-26 起**——grep 确认 UT-MM-25 已被 ux-canvas-batch 批次 2 收尾占用）
> **C-3 裁决（外环代决，R3 技术取舍）**：导出**降格为仅 CSV**，纯手写无依赖；不引入 xlsx（zip 二进制 + wasm 体积代价，YAGNI）

## [code] 代码实现（批次 3：批量改类型 + 双击跳画布 + 导出仅 CSV）

### 批量改类型（多字段一次性改类型）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `batch_change_types` 纯函数（UT-MM-26）：
  - 输入：`fields: &mut Vec<Field>`、`type_map: HashMap<String, String>`（字段 ID → 新类型）
  - 规则（**类型兼容性边界真值表**，外环判词 C-1 强制 + 强类型边界）：

    | 源类型 | 目标类型 | 兼容性 | 结果 | 备注 |
    |---|---|---|---|---|
    | INT | INT | 同型 | 直接改 | — |
    | INT | BIGINT | 兼容（数值收窄反向——目标更宽） | 直接改 | — |
    | INT | VARCHAR | 兼容（需长度声明，否则 VARCHAR(255) 默认） | 直接改 + 默认长度补全 | — |
    | INT | DECIMAL | 兼容（数值→数值） | 直接改 | — |
    | VARCHAR | INT | **不兼容**（字符串→数值） | 跳过该字段 | 需手动迁移 |
    | VARCHAR | VARCHAR(50) | 兼容（精度收窄——需确认） | 跳过该字段（精度收窄需用户确认，不静默执行） | C-1 边界 |
    | BOOLEAN | INT | 兼容（0/1 映射） | 直接改 | — |
    | DATE | DATETIME | 兼容（精度扩展） | 直接改 | — |
    | （任意）| "" | **非法目标类型** | 跳过该字段 | — |
    | （任意）| "INVALID_TYPE" | **非法目标类型**（不在 enum 内） | 跳过该字段 | — |

  - **真值表语义**：
    - 兼容（数值收窄反向、字符串可表达、布尔映射、日期精度扩展）：直接改
    - 不兼容（字符串→数值）：跳过（需手动迁移）
    - 精度收窄（VARCHAR(50) ← VARCHAR(255)）：**跳过**（精度收窄需用户确认，不静默执行——C-1 类型兼容性边界）
    - 非法目标类型（空、不在 enum 内）：**跳过**
  - 批量改名后 `store.dirty.set(true)`（标记脏，触发自动保存）
  - **实例推演**（外环判词 C-1 强制）：
    - 场景 1：fields = [id(INT), name(VARCHAR(255))], type_map = {id→BIGINT, name→INT} → id→BIGINT(改), name→跳过(VARCHAR→INT 不兼容)
    - 场景 2：fields = [id(INT), name(VARCHAR(255))], type_map = {id→INT, name→VARCHAR(50)} → id→INT(改), name→跳过(精度收窄 VARCHAR(50)←VARCHAR(255))
    - 场景 3：fields = [id(INT)], type_map = {id→"INVALID_TYPE"} → id→跳过(非法目标类型)
    - 场景 4：fields = [id(INT), name(VARCHAR(255))], type_map = {id→VARCHAR} → id→VARCHAR(改, 默认长度 VARCHAR(255))
    - 场景 5：fields = [id(BOOLEAN), name(VARCHAR(255))], type_map = {id→INT, name→DECIMAL} → id→INT(改, 0/1 映射), name→DECIMAL(改, 字符串→数值——**不兼容，应跳过**——修正场景 5)

### 双击跳画布对应表

- [ ] `frontend-rs/src/editor_panels.rs`: ListView 组件内双击行（`on:dblclick`）调 `on_select_table(Some(table_id.clone()))`
  - 跳转后**同时切回 `ViewMode::Canvas`**（外环条目 10 强制）：List 态下双击跳到画布对应表 → 切回画布 → 选中态表达走既有 `on_select_table(Some(table_id.clone()))` 通路（批次 1 ListView 已用）
  - 实现方式：ListView 需新增 `on_jump_to_canvas: Rc<dyn Fn()>` prop，AppRoot 调用点传 `move || view_mode.set(ViewMode::Canvas)`
  - data-testid 命名：`list-view-dblclick-{table_id}`（外环判词建议沿用既有规范；或保留 on:click + 加 on:dblclick 共存，**禁止改既有 data-testid**）

### 导出仅 CSV（外环 C-3 裁决）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `export_tables_csv` 纯函数（UT-MM-27）：
  - **不引入 xlsx 依赖**（YAGNI，wasm 体积代价）——纯手写 CSV 序列化
  - **CSV 转义规则真值表**（外环判词 C-3 强制）：

    | 字符 | 在字段值中 | CSV 输出 | 实例推演 |
    |---|---|---|---|
    | `,`（逗号）| 是 | 字段加双引号 `"..."` | `apple,banana` → `"apple,banana"` |
    | `"`（双引号）| 是 | 字段加双引号 + 内部双引号转义为 `""` | `she said "hi"` → `"she said ""hi"""` |
    | `\n`（换行）| 是 | 字段加双引号 `"..."` | `line1\nline2` → `"line1\nline2"` |
    | 三者均无 | 否 | 字段不加引号 | `apple` → `apple` |
    | 字段为空 | — | 空字段 | `apple,,banana` → `apple,,banana` |
    | 字段为 NULL | — | 空字段 | `apple,,banana` → `apple,,banana` |

  - **实例推演**（外环判词 C-3 强制）：
    - 场景 1：表 [id(INT)=1, name(VARCHAR(255))=apple] → `1,apple`
    - 场景 2：表 [id=1, name=apple,banana] → `1,"apple,banana"`
    - 场景 3：表 [id=1, name=she said "hi"] → `1,"she said ""hi"""`
    - 场景 4：表 [id=1, name=line1\nline2] → `1,"line1\nline2"`
    - 场景 5：表 [id=1, name=NULL] → `1,`
  - 表头行：每个字段名作为 CSV 第一行
- [ ] `frontend-rs/src/editor_panels.rs`: 导出 CSV UI（ListView 组件内）：
  - 导出 CSV 按钮：data-testid `list-view-export-csv`
  - 弹出下载（前端用 `Blob` + `URL.createObjectURL` + `<a download>` 触发下载）：data-testid `list-view-export-csv-download`

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-26）：
  - happy: `batch_change_types(fields, {id→BIGINT})` → id→BIGINT(改)
  - happy: `batch_change_types(fields, {id→INT})` → id→INT(同型改)
  - happy: `batch_change_types(fields, {id→VARCHAR})` → id→VARCHAR(默认 VARCHAR(255))
  - happy: `batch_change_types(fields, {id→VARCHAR(50)})` → id→跳过(精度收窄)
  - edge: `batch_change_types(fields, {id→"INVALID_TYPE"})` → id→跳过(非法目标类型)
  - edge: `batch_change_types(fields, {name→INT})` → name→跳过(VARCHAR→INT 不兼容)
  - edge: `batch_change_types(fields, {id→""})` → id→跳过(空字符串)
  - edge: `batch_change_types(fields, {})` → 全部不变(空 type_map)
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-27）：
  - happy: `export_tables_csv([(id, apple)])` → `id\n1,apple`
  - happy: `export_tables_csv([(id, "apple,banana")])` → `1,"apple,banana"`（逗号转义）
  - happy: `export_tables_csv([(id, 'she said "hi"')])` → `1,"she said ""hi"""`（引号转义）
  - happy: `export_tables_csv([(id, "line1\nline2")])` → `1,"line1\nline2"`（换行转义）
  - edge: `export_tables_csv([])` → `id\n`（空表头）
  - edge: `export_tables_csv([(id, "")])` → `1,`（空字段）
  - edge: `export_tables_csv([(id, "apple")])` → `1,apple`（无转义字符）

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-26/27 行：
  ```
  | UT-MM-26 | 列表视图批量改类型纯函数测试（类型兼容性边界：数值收窄反向/字符串可表达/精度收窄需确认/不兼容/非法目标类型） | `editor_panels.rs::batch_change_types` |
  | UT-MM-27 | 列表视图导出 CSV 纯函数测试（CSV 转义：逗号/引号/换行三字符——C-3 裁决仅 CSV 纯手写无依赖） | `editor_panels.rs::export_tables_csv` |
  ```
- [ ] 确认 UT-MM-26/27 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议（批次 3）

1. `batch_change_types` 纯函数 + UT-MM-26（8 子用例：类型兼容性边界真值表覆盖）
2. 双击跳画布对应表（ListView 加 `on_jump_to_canvas` prop + `on:dblclick` + AppRoot 调用点切回 Canvas）
3. `export_tables_csv` 纯函数 + UT-MM-27（7 子用例：CSV 转义真值表覆盖）
4. 导出 CSV UI（ListView 组件内：导出按钮 + 下载触发）
5. spec 登记（UT-MM-26/27 行）

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

## 外环判词强制约束落实

- **C-1**：批量改类型类型兼容性边界真值表 + 实例推演已给出（见 `batch_change_types` 纯函数段，8 子用例覆盖所有边界）
- **C-3 裁决**：导出仅 CSV 纯手写无依赖；不引入 xlsx；CSV 转义真值表 + 实例推演已给出（见 `export_tables_csv` 纯函数段，7 子用例覆盖逗号/引号/换行三字符）
- **外环条目 10 强制要求**：双击跳画布须切回 `ViewMode::Canvas` + 选中态走既有 `on_select_table(Some(table_id.clone()))` 通路——已落实
- **新 UT 编号**：UT-MM-26/27（grep 确认 UT-MM-25 已被 ux-canvas-batch 批次 2 收尾占用，UT-MM-26 起编号空闲）