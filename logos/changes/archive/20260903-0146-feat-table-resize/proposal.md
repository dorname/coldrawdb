# 变更提案：feat-table-resize

> module: core | created: 2026-09-02
> Guard: `logos/.openlogos-guard` 指向 `feat-table-resize`（commit `4d87a1b` 外环代行开案）
> 内容基线：`.octos/proposals/draft-2026-09-02-product-batch/04-feat-table-resize-proposal.md` v2（commit `db57087`）
> 上游裁决：黑板 `.octos/OUTER_LOOP_REVIEW.md` 条目6 operator 批注 + 外环 R2 复核采认（commit `8bf2500` 起两次修订）

## 变更原因

**现状 Bug + UX 缺口**（事实层 R2 复核通过）：

- `frontend-rs/src/editor_core.rs:43` `Table` struct 无 `width` 字段；`:103` 的 `pub width: f64` 属 `Area` struct（与本提案无关）
- `frontend-rs/src/editor_render.rs:18-20` 渲染用硬编码常量 `TABLE_WIDTH=230.0`、`TABLE_HEADER_HEIGHT=43.0`、`FIELD_ROW_HEIGHT=35.0`
- `frontend-rs/src/editor_panels.rs:8138+` `SetTableWidthModal` 是模态（不是 Inspector），**Apply 按钮 `modal-submit-set-width` 无 `on:click`**——宽度链路 UI 是空壳，未闭环到 store
- `frontend-rs/src/editor_panels.rs:7473` `parse_table_width(input: &str) -> Result<u32, String>`：接受 `"0" → Ok(0)`（"0 = auto"，u32 parse 不带范围限制）
- 表 record **无高度字段**（grep `parse_table_height` 无命中），字段多时无最小高度控制

**operator 裁决（黑板条目6 Q4）**：采用**最小高度语义**——用户设最小值，字段多时自动撑高。`render_height = max(min_height.unwrap_or(default), TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT × field_count)`。

## 变更类型

**代码级修复**（参考 `spec/tasks-spec.md` 与 `logos/skills/change-writer/SKILL.md` Step 3 判定）：
- 影响的 PRD/API/DB schema：**无**
- 影响的功能规格：**无**（字段 id 生成规则不是需求级事实；`auto-N`/`width` 等不属规格字面量断言）
- 影响的部署方案：**无**（纯前端 WASM）
- 影响的 smoke：**无**

故 `tasks.md` 采用**代码级修复模板**（无 `[delta]`、`[deploy]` section）。

## 变更范围

- 影响的需求文档：**无**（高度字段不属于需求级事实）
- 影响的功能规格：**无**（grep 既有规格文档无 `Table.width` / `Table.min_height` 字面量断言）
- 影响的业务场景：
  - S01（编辑并保存 diagram）：高度字段加入 store → 保存链路需携带（已是 JSON blob 全量保存，自动适配）
  - S05（OT 实时协作）：width/min_height 字段加入后，OT 操作需包含此字段——实现时验证 op 应用器对未声明字段的处理
- 影响的部署方案：**无**
- 影响的 API：**无**（`PUT /api/v1/diagrams/{id}` 契约不变，JSON blob 内字段增减对后端透明）
- 影响的 DB 表：**无 schema 变更**（diagram 整体存 JSON blob，前端字段加 DB 不变；存量数据无 `width`/`min_height` 字段，serde 反序列化为 `None`，渲染走硬编码 fallback）
- 影响的编排测试：场景 S01/S05 的下游测试需验证新字段不破坏断言
- 影响的 smoke 测试：**无**

## 部署影响

- 是否需要部署：**否**
- 部署原因：纯前端 WASM 代码修复，本地开发环境重新构建即生效；当前项目处于开发阶段，无独立部署节点
- 影响环境：**无**
- 是否涉及数据迁移：**否**（存量数据无 `width`/`min_height` 字段，serde 默认 `None`，渲染走硬编码 `TABLE_WIDTH=230.0` fallback，视觉无突变）
- 是否需要回滚预案：**否**（小切片，回滚 = revert commit）
- 是否需要 smoke：**否**

## 变更概述

给 `Table` struct 同时新增 `width: Option<u32>` 和 `min_height: Option<u32>` 两个字段，并让 `editor_render.rs::draw_table`（:1166）等渲染路径消费这两个字段替代硬编码常量 `TABLE_WIDTH=230.0`。新增 `parse_table_height` 纯函数（严格对称 `parse_table_width` 的 `"0" → Ok(0)` 语义）。补全 `SetTableWidthModal` Apply 按钮的 `on:click` handler 让宽度链路闭环到 store。`hit_test_field`（:1427）与 `hit_test`（:1450）的命中测试同步改为消费 `table.width`，避免新宽度下命中区域错位。`min_height` 入口采用模态（与宽度对称；统一模态 vs 拆双模态实现时定）。

## 设计决策记录（ADR-style 摘要）

| 决策 | 选 | 否 | 依据 |
|---|---|---|---|
| 高度语义 | 最小高度（auto 撑高）| 绝对高度 / 完全自动 | operator Q4 裁决 |
| 字段放法 | `width` + `min_height` 平铺 | 合并为 `size: { w, h }` 对象 | 向后兼容 + Q9 模板原则 |
| 入口 | 模态 | Inspector 输入框 | `SetTableWidthModal` 现状是模态；新对称一致 |
| 渲染常量 | 复用 `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0` | 新建 `DEFAULT_MIN_HEIGHT` | v1 草案错误，外环 R2 修正 |
| 0 值语义 | `"0" → Ok(0)`（auto）| `"0" → Err` | 对称 `parse_table_width` |

## 范围外（明确排除）

- 不修改 Inspector 任何字段（v1 草案错误地把模态当 Inspector；v2 已纠正）
- 不修改表的固定行高常量（属样式优化，归 `ux-canvas-batch` C 案）
- 不加拖拽 resize 手柄（属 UX 大改动，归 C 案）
- 不改字段顺序拖拽（属需求 6，归 `ux-ergonomics-subset` E 案）
- 不引入 reference 连线布局的端点重计算算法（仅跟随 `width` 变化做端点位置平移；新算法属 `feat-relation-inference` D 案范畴）

## 风险点

- **R1**：`draw_table` / `hit_test_field` / `hit_test` 渲染消费 `table.width`，reference 端点计算（:935）`table.x + TABLE_WIDTH` 改为 `table.x + width`，可能让 reference 连线端点偏移——需在 Canvas draw 与 reference 重算函数之间加联调测试
- **R2**：模态视觉与现有 `ST-FE-ALIGN-*` 测试视觉断言不能破坏
- **R3**：`Table.width = None` 时 fallback 到 `TABLE_WIDTH=230.0` 硬编码——若实现时把 fallback 常量定义错位置会出现常量重复
- **R4**：与未来 `feat-relation-inference` D 案的端点计算可能耦合（reference 端点跟随 width 已在本提案做最小修改，但 D 案的端点重计算算法会再次触碰）
- **R5**：OT 操作需携带 `width` 和 `min_height`（新增字段），需检查 op 应用器对未声明字段的处理（推测 silently ignore，实现时验证）

## 替代方案否决理由

- **A 绝对高度**：字段多时溢出、少时空洞——否决（与 Q4 operator 裁决相反）
- **B 完全自动**：等于零工作量不算新功能——否决（operator 显式要求用户可控）
- **C 合并为 `size: { w, h }` 对象**：破坏向后兼容——否决（Q9 原则：宽度复用既有 `parse_table_width` 路径）
- **D 合并到 `ux-canvas-batch` C 案**：增 C 案工作量至 10+ 天，违反"5 案串行"节奏——否决
- **E 只做 `min_height` 不做 `width`**：v1 草案范围；外环 R2 已批驳（`SetTableWidthModal` Apply 无 on:click 是空壳 + 渲染消费硬编码常量必须修复）——否决

## 关联场景

- **S01（编辑并保存图表）**：width/min_height 加入 store → 保存链路（已是 JSON blob 全量保存，自动适配）
- **S05（OT 实时协作）**：width/min_height 加入后，OT 操作需包含此字段——实现时验证 op 应用器对未知字段的处理
- **S02（分享链接图表）**：老数据 width/min_height = None 走硬编码 fallback，分享出去的图表视觉无突变
