# 变更提案：ux-canvas-batch

> module: core | created: 2026-09-02
> Guard: `logos/.openlogos-guard` 指向 `ux-canvas-batch`（外环代行开案）
> 内容基线：`.octos/proposals/draft-2026-09-02-product-batch/08-ux-canvas-batch-proposal.md`（commit `905558e`）
> 上游裁决链：黑板条目6 operator 批注（Q1=全部 9 项列表视图候选；Q5=外环代决样式子集）→ 外环判词（采认 + 执行条件 C-1..C-4 + 记一笔「按任意列」→「按表维度属性排序」）→ 外环代行开案
> 工作量档位：8 天档（operator Q1 全选列表视图全量能力）

## 变更原因

**UX 缺口**（operator 原始需求 1+5）：
1. **需求 1**：表结构列表视图（参考 pdmaner）——当前编辑器只有画布视图（HTML5 Canvas 自绘），无跨表全量列表视图；Inspector 是单表焦点视图（field 列表 + reference 列表），但无跨表全量列表
2. **需求 5**：样式优化（字体清晰度、交互流畅性）——当前样式有基础（`styles.css:113-115` 字体回退栈 + `:366-368` 子像素抗锯齿已存在），但 Canvas 文本绘制未走离屏缓存、关键交互帧率未量化

**现状事实层**（外环判词亲测，非转述内环清单）：

- **画布渲染常量与文本绘制路径**（`editor_render.rs`）：
  - `frontend-rs/src/editor_render.rs:18-20` 常量 `TABLE_WIDTH=230.0`、`TABLE_HEADER_HEIGHT=43.0`、`FIELD_ROW_HEIGHT=35.0`（feat-table-resize 已消费 `table.width`/`table.min_height` 替代硬编码 TABLE_WIDTH，但 TABLE_HEADER_HEIGHT/FIELD_ROW_HEIGHT 仍硬编码）
  - `frontend-rs/src/editor_render.rs:38-67` 字体函数族：`dpr_font_boost`（DPR 缩放字号）、`dpr_font`（组装字号字符串）、`resolve_canvas_font_family`（探测 Plus Jakarta Sans 是否真正可用，不可用降级 ui-monospace）
  - `frontend-rs/src/editor_render.rs:63-65` `fonts.check(&format!("1em \"{}\"", primary))` —— Canvas 文本绘制用 `doc.fonts().check()` 探测字体加载状态
- **侧栏结构**（`editor_panels.rs`）：
  - `frontend-rs/src/editor_panels.rs:215-249` `SidePanelTab` enum 含 8 个 tab（Tables/Areas/Enums/Notes/Relationships/Types/Issues/Fields）——**无 ListView tab**（新增）
  - `frontend-rs/src/editor_panels.rs:229-249` `SidePanelTab::testid()`/`label()` 映射——新增 `ListView` tab 需补 testid/label
- **字体加载现状**（`styles.css`/`index.html`）：
  - `frontend-rs/src/styles.css:113-115` 字体回退栈已存在（`--cdb-font-family-base: "Plus Jakarta Sans", -apple-system, ...`）
  - `frontend-rs/src/styles.css:366-368` 子像素抗锯齿已存在（`-webkit-font-smoothing: antialiased`）
  - `frontend-rs/index.html:10` `<link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700&display=optional" rel="stylesheet">` —— Plus Jakarta Sans 通过 Google Fonts CDN 加载
  - **无中文字体加载**（思源黑体/苹方未在 index.html 中加载；`PingFang SC`/`Hiragino Sans GB`/`Microsoft YaHei` 在 `--cdb-font-family-base` 回退栈中但未显式加载）

**operator 裁决（Q1 + Q5）**：
- Q1：**全部 9 项列表视图候选**都做（全量列表视图能力）——表名/字段名/类型表格化展示、排序（**按表维度属性排序**——外环判词记一笔修正措辞：「按任意列」实为「按表维度属性排序」，避免实现期误解为仅展示列可排序）、过滤（按名称模糊匹配/按类型/按是否有索引）、批量重命名（多表或多字段一次性改名）、批量改类型（多字段一次性改类型）、双击跳到画布对应表、导出 CSV/Excel、列宽可调、表/字段分组（按 schema/按 tag）
- Q5：**外环代决样式子集**：字体回退栈+子像素抗锯齿+中文字体思源黑体/苹方+Canvas 文本离屏缓存+关键交互<16ms+rAF 统一调度；大图（>200 表）虚拟化**暂缓**（性能专项另立提案）

**外环判词执行条件（C-1..C-4）**：
- **C-1**：批次 2/3/4 派发前必须补齐该批细化 tasks；凡涉及规则推导的（批量重命名的**重名冲突处理**、批量改类型的**类型兼容性边界**、CSV 导出的**转义规则**）必须给真值表或明确规则 + 实例推演，不得以「实现时定」留白
- **C-2**：「关键交互帧率 < 16ms」**不得**作为 verify 门禁断言（CI 无可靠帧率测量手段，写入门禁必翻车）——降为代码审查项 + 可选基准脚本；Canvas 离屏缓存以现有视觉回归 ST（ST-FE-ALIGN-* 等）全绿为验收标准
- **C-3**：「导出 CSV/Excel」实现方式在批次 3 派发时明确：CSV 可纯手写无依赖；xlsx 是 zip 二进制格式，要么明确引入依赖（评估 wasm 体积代价）要么降格为仅 CSV——二选一写进 tasks，不允许模糊
- **C-4**：`ListViewState` 落点不一致（proposal 写 `editor_core.rs`，tasks 批次 1 写 `editor_panels.rs`）——实现时统一落点并在正式版修正，以纯函数可测性为准

## 变更类型

**代码级修复**（参考 `spec/tasks-spec.md` 与 `logos/skills/change-writer/SKILL.md` Step 3 判定）：
- 影响的 PRD/API/DB schema：**无**（列表视图是前端呈现层，无 schema 变更）
- 影响的功能规格：**无**（列表视图功能是需求级事实，但既有规格无列表视图字面量断言）
- 影响的部署方案：**无**（纯前端 WASM）
- 影响的 smoke：**无**

故 `tasks.md` 采用**代码级修复模板**（无 `[delta]`、`[deploy]` section）。

## 变更范围

- 影响的需求文档：**无**（列表视图功能是需求级事实，但既有规格无列表视图字面量断言）
- 影响的功能规格：**无**（grep 既有规格文档无列表视图字面量断言）
- 影响的业务场景：
  - S01（编辑并保存 diagram）：列表视图编辑后落 store → 保存链路需携带（已是 JSON blob 全量保存，自动适配）
  - S05（OT 实时协作）：列表视图编辑后 OT 操作需包含编辑结果——实现时验证 op 应用器对字段的处理
- 影响的部署方案：**无**
- 影响的 API：**无**（`PUT /api/v1/diagrams/{id}` 契约不变，JSON blob 内字段增减对后端透明）
- 影响的 DB 表：**无 schema 变更**（diagram 整体存 JSON blob，前端列表视图是呈现层，无 schema 变更）
- 影响的编排测试：场景 S01/S05 的下游测试需验证列表视图编辑不破坏断言
- 影响的 smoke 测试：**无**

**代码影响面**（`frontend-rs/`）：
- `src/editor_panels.rs`：
  - `SidePanelTab:215-249` 新增 `ListView` tab（testid/label 补全）
  - 新增 `ListView` 组件（表结构列表视图：表名/字段名/类型表格化展示 + **按表维度属性排序** + 过滤 + 批量重命名 + 批量改类型 + 双击跳到画布对应表 + 导出 CSV/Excel + 列宽可调 + 表/字段分组）
  - 侧栏 tab 切换逻辑（ListView tab 激活时显示列表视图）
  - **ListViewState 落点统一**（C-4：实现时统一落点并在正式版修正，以纯函数可测性为准——本提案正式版统一落 `editor_panels.rs`）
- `src/styles.css`：
  - 字体回退栈补思源黑体/苹方显式加载（`index.html` 加 `<link>` 加载思源黑体/苹方）
  - Canvas 文本离屏缓存（`editor_render.rs` 文本绘制路径优化：离屏 canvas 缓存 + 帧率 < 16ms + rAF 统一调度）
- `src/editor_render.rs`：
  - Canvas 文本离屏缓存（`draw_table`/`draw_field`/`draw_reference` 等文本绘制路径优化：离屏 canvas 缓存 + 帧率 < 16ms + rAF 统一调度）
  - 关键交互帧率 < 16ms（拖拽/连线/Inspector 切换）——**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**
  - rAF 统一调度（requestAnimationFrame 统一调度，避免重复渲染）
- `src/editor_core.rs`：
  - store 派生 selectors（列表视图数据派生：表/字段/类型/索引/备注 等）
  - **ListViewState 落点统一**（C-4：实现时统一落点并在正式版修正，以纯函数可测性为准——本提案正式版统一落 `editor_panels.rs`）

## 部署影响

- 是否需要部署：**否**
- 部署原因：纯前端 WASM 代码修复，本地开发环境重新构建即生效；当前项目处于开发阶段，无独立部署节点
- 影响环境：**无**
- 是否涉及数据迁移：**否**（列表视图是前端呈现层，无 schema 变更；存量数据原字段保留）
- 是否需要回滚预案：**否**（大切片，回滚 = revert commit；但需评估回滚对列表视图数据的影响）
- 是否需要 smoke：**否**

## 变更概述

新增**表结构列表视图**（参考 pdmaner 全量能力）+ **样式优化**（字体清晰度、交互流畅性）：
- **列表视图**：SidePanelTab 新增 `ListView` tab，表名/字段名/类型表格化展示 + **按表维度属性排序** + 过滤 + 批量重命名 + 批量改类型 + 双击跳到画布对应表 + 导出 CSV/Excel + 列宽可调 + 表/字段分组
- **样式优化**：字体回退栈补思源黑体/苹方显式加载 + Canvas 文本离屏缓存 + 关键交互帧率 < 16ms（**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**）+ rAF 统一调度（大图虚拟化暂缓）
- **范围大（8 天档）**：草案自带分批建议（见下方"分批建议"段）

## 分批建议（8 天档，外环 steer 可分批派发）

| 批次 | 范围 | 工作量 | 依赖 |
|---|---|---|---|
| 批次 1 | SidePanelTab 新增 `ListView` tab + 基础表格化展示（表名/字段名/类型）+ **按表维度属性排序** | 2 天 | 无 |
| 批次 2 | 过滤（按名称模糊匹配/按类型/按是否有索引）+ 批量重命名（多表或多字段一次性改名）——**C-1：批量重命名的重名冲突处理必须给真值表或明确规则 + 实例推演** | 2 天 | 批次 1 |
| 批次 3 | 批量改类型（多字段一次性改类型）——**C-1：批量改类型的类型兼容性边界必须给真值表或明确规则 + 实例推演**；双击跳到画布对应表 + 导出 CSV/Excel——**C-3：CSV 可纯手写无依赖；xlsx 是 zip 二进制格式，要么明确引入依赖（评估 wasm 体积代价）要么降格为仅 CSV——二选一写进 tasks，不允许模糊** | 2 天 | 批次 1 |
| 批次 4 | 列宽可调 + 表/字段分组（按 schema/按 tag）+ 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + 关键交互帧率 < 16ms（**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**）+ rAF 统一调度）| 2 天 | 批次 1/2/3 |

**总工作量**：8 天（operator Q1 全选列表视图全量能力）

## 设计决策记录（ADR-style 摘要）

| 决策 | 选 | 否 | 依据 |
|---|---|---|---|
| 列表视图能力 | 全部 9 项（operator Q1 全选）| 部分子集 | operator Q1 裁决 |
| 样式子集 | 字体回退栈+子像素抗锯齿+中文字体+Canvas 离屏缓存+16ms+rAF | 大图虚拟化 | operator Q5 裁决（虚拟化暂缓）|
| 排序措辞 | **按表维度属性排序** | 按任意列 | 外环判词记一笔修正措辞（避免实现期误解为仅展示列可排序）|
| 数据结构 | `ListViewState`（排序列/过滤条件/分组方式等）| 扩展 `Reference`/`Table` struct | 列表视图是呈现层，无 schema 变更 |
| `ListViewState` 落点 | `editor_panels.rs`（**C-4：实现时统一落点并在正式版修正，以纯函数可测性为准**）| `editor_core.rs`（proposal 原写）| 外环判词 C-4 修正 |
| 分批策略 | 4 批次（基础表格化 → 过滤/批量重命名 → 批量改类型/导出 → 列宽/分组/样式优化）| 单批次 8 天 | 范围大，分批降低单提案复杂度 |
| CSV/Excel 实现方式 | **批次 3 派发时明确**（**C-3：CSV 可纯手写无依赖；xlsx 是 zip 二进制格式，要么明确引入依赖（评估 wasm 体积代价）要么降格为仅 CSV——二选一写进 tasks，不允许模糊**）| 模糊实现 | 外环判词 C-3 强制 |
| 帧率 < 16ms | **降为代码审查项 + 可选基准脚本**（**C-2：不作为 verify 门禁断言**）| verify 门禁断言 | 外环判词 C-2 强制（CI 无可靠帧率测量手段）|

## 真值表（D 案教训：涉及推导/状态机必须给真值表+实例推演）

**列表视图排序规则**（**按表维度属性排序**——外环判词记一笔修正措辞）：

| 排序列 | 排序方向 | 结果 |
|---|---|---|
| 表名 | 升序 | 按表名字典序升序 |
| 表名 | 降序 | 按表名字典序降序 |
| 字段数 | 升序 | 按字段数升序（少→多）|
| 字段数 | 降序 | 按字段数降序（多→少）|
| 类型 | 升序 | 按类型字典序升序（如 INT < VARCHAR）|
| 是否有索引 | 升序 | 无索引 → 有索引 |
| 是否有索引 | 降序 | 有索引 → 无索引 |

**实例推演**（排序规则）：
- 表 A（5 字段，有索引）、表 B（3 字段，无索引）、表 C（10 字段，有索引）
- 按字段数升序：B(3) → A(5) → C(10)
- 按字段数降序：C(10) → A(5) → B(3)
- 按是否有索引降序：A(有) → C(有) → B(无)

## 范围外（明确排除）

- 大图（>200 表）虚拟化——operator Q5 裁决暂缓（性能专项另立提案）
- 不修改 `Reference`/`Table` struct 数据契约（列表视图是呈现层，无 schema 变更）
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）

## 风险点

- **R1**：列表视图与 Inspector 数据同步——列表视图编辑后 Inspector 需同步更新（如批量重命名表名后 Inspector 表名输入框需同步）
- **R2**：Canvas 文本离屏缓存可能破坏现有视觉断言（ST-FE-ALIGN-* 等）——需回归测试（**C-2：Canvas 离屏缓存以现有视觉回归 ST 全绿为验收标准**）
- **R3**：关键交互帧率 < 16ms 需性能测试（非功能测试，需性能基准）——**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**
- **R4**：与 feat-table-resize 的渲染消费可能耦合（列表视图需消费 `table.width`/`table.min_height` 字段）
- **R5**：与 feat-relation-inference 的 cardinality 推导可能耦合（列表视图需显示 cardinality 推导结果）

## 替代方案否决理由

- **A 部分子集（如只做排序+过滤）**：operator Q1 全选 9 项——否决（operator 裁决优先）
- **B 单批次 8 天**：范围大，单提案复杂度高——否决（分批降低单提案复杂度）
- **C 大图虚拟化纳入**：operator Q5 裁决暂缓——否决（operator 裁决优先）
- **D 扩展 `Reference`/`Table` struct**：破坏向后兼容——否决（列表视图是呈现层，无 schema 变更）
- **E 按任意列**：外环判词记一笔修正措辞——否决（改为「按表维度属性排序」，避免实现期误解为仅展示列可排序）
- **F 帧率 < 16ms 写入门禁**：外环判词 C-2 强制——否决（降为代码审查项 + 可选基准脚本）
- **G CSV/Excel 模糊实现**：外环判词 C-3 强制——否决（批次 3 派发时明确二选一写进 tasks）

## 关联场景

- **S01（编辑并保存图表）**：列表视图编辑后落 store → 保存链路（已是 JSON blob 全量保存，自动适配）
- **S05（OT 实时协作）**：列表视图编辑后 OT 操作需包含编辑结果——实现时验证 op 应用器对字段的处理（`CommandStack::apply` 直接接收完整 `Table`/`Field` 对象，随 Table/Field struct 序列化自动携带）

## 关联任务清单

见 `tasks.md`。

## 不在范围（明确排除）

- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 不修改 `Reference`/`Table` struct 数据契约
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）