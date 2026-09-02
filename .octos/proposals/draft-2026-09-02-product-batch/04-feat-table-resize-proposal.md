# 变更提案：feat-table-resize

> module: core | created: 2026-09-02
> 状态：**草案 v2（外环条目6 切片2 判词打回修订后重写）**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 父批次：[产品优化批次-2026-09-02](./01-current-state.md) 切片 2 / 首案
> 裁决依据：黑板条目6 operator 批注（Q4=最小高度语义；Q9=完整模板）+ 外环 R2 事实复核 4+2 修正

## 变更原因

**线上 Bug + UX 缺口（事实层 R2 复核通过）**：
- `frontend-rs/src/editor_core.rs:43` `Table` struct **无 width 字段**（仅有 id/name/x/y/color/comment/fields）
- `frontend-rs/src/editor_core.rs:103` 的 `pub width: f64` 属 `Area` struct（与本提案无关）
- `frontend-rs/src/editor_render.rs:18-20` 渲染用**硬编码常量** `TABLE_WIDTH=230.0`、`TABLE_HEADER_HEIGHT=43.0`、`FIELD_ROW_HEIGHT=35.0`
- `frontend-rs/src/editor_panels.rs:8138+` `SetTableWidthModal` 是模态而非 Inspector；**Apply 按钮无 on:click**（仅 `disabled=move || !is_valid()`）—— 宽度链路 UI 是空壳，未闭环到 store
- `frontend-rs/src/editor_panels.rs:7473` `parse_table_width(input: &str) -> Result<u32, String>`：**接受 0**（"0 = auto"，u32 parse 不带范围限制，注释明确 `"0" → Ok(0)`）

**operator 裁决（Q4）**：采用**最小高度语义**——用户设最小值，字段多时自动撑高。

**修正后的提案范围（v2 vs v1）**：v1 错把 width 当"既有字段"，实际 width **从未落到 `Table` 上**，渲染消费的是硬编码常量 `TABLE_WIDTH=230.0`。本提案正确范围：**给 `Table` 同时新增 `width` 和 `min_height` 两个字段**，并让 `editor_render.rs::draw_table` 消费这两个字段替代硬编码常量。

## 变更类型

代码级修复 + UX 增强（中度）—— 范围比 v1 草案大一倍，但仍是独立可交付切片

## 变更范围

### Why

让用户控制表的**宽度**与**最小高度**：
- 宽度：当前 `SetTableWidthModal` 是空壳（Apply 无 on:click），需补全 on:click 让宽度真正落 store，并被渲染消费
- 高度：高度字段不存在；用最小高度语义（Q4 operator 裁决），让用户控制"紧凑 vs 宽松"
- 渲染：当前 `draw_table` 用硬编码 `TABLE_WIDTH=230.0` + `TABLE_HEADER_HEIGHT+FIELD_ROW_HEIGHT*field_count`，新字段必须**替换**这两个常量才能生效

### What

1. `frontend-rs/src/editor_core.rs`: `Table` struct 新增 `width: Option<u32>`（替代硬编码 `TABLE_WIDTH`）+ `min_height: Option<u32>`（最小高度语义）
2. `frontend-rs/src/editor_panels.rs`: 新增 `pub fn parse_table_height(input: &str) -> Result<u32, String>`，**对称 `parse_table_width` 的 0 = auto 语义**：`""` → Err；其它（**含 0**）→ Ok(u32)
3. `frontend-rs/src/editor_panels.rs`: `SetTableWidthModal` Apply 按钮补 `on:click` handler，把 `width_input` 值写入选中 table 的 `width` 字段
4. `frontend-rs/src/editor_panels.rs`: 新增 `SetTableMinHeightModal`（**或扩展 `SetTableWidthModal` 为 `SetTableSizeModal` 含 width + min_height** —— 实现时定）
5. `frontend-rs/src/editor_render.rs`: `draw_table` 函数把硬编码 `TABLE_WIDTH=230.0` 替换为 `table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH_DEFAULT)`；高度计算改为 `let total_height = max(table.min_height.map(|h| h as f64).unwrap_or(MIN_HEIGHT_DEFAULT), TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64)`
6. `frontend-rs/src/editor_panels.rs`: 复用现有 `TABLE_HEADER_HEIGHT` / `FIELD_ROW_HEIGHT` 常量**（不在 `editor_render.rs` 模块内私有，但在 `editor_panels.rs` 也不存在同名常量——本提案新增渲染常量时复用这两个，**不**新建 `DEFAULT_MIN_HEIGHT` / `ROW_HEIGHT`）**

### 数据契约变更

```rust
// frontend-rs/src/editor_core.rs:43 Table struct
pub struct Table {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub color: String,
    pub comment: String,
    pub fields: Vec<Field>,
    // ... indices 等既有字段不动
    pub width: Option<u32>,       // ← 新增（None = 用 TABLE_WIDTH_DEFAULT=230.0 硬编码常量）
    pub min_height: Option<u32>,  // ← 新增（None = 不强制最小，按字段数自动撑高）
}
```

**向后兼容**：
- 老 JSON 无 `width`/`min_height` 字段 → serde 反序列化为 `None` → 渲染 fallback 到 `TABLE_WIDTH=230.0` 硬编码（与现状视觉一致，无突变）
- 老数据无需 migration
- DB schema 不变（diagram 整体存 JSON blob，前端字段加 DB 不变）

### 文件影响

| 文件 | 改动 |
|---|---|
| `frontend-rs/src/editor_core.rs` | `Table` struct 新增 `width: Option<u32>` + `min_height: Option<u32>` |
| `frontend-rs/src/editor_panels.rs` | 新增 `parse_table_height`；`SetTableWidthModal` Apply 补 on:click；新增 `SetTableMinHeightModal` 或扩展为 `SetTableSizeModal` |
| `frontend-rs/src/editor_render.rs` | `draw_table` 函数消费 `table.width` / `table.min_height`，替代硬编码 `TABLE_WIDTH`；高度计算引入 `max(min_height, fields×ROW)`；复用现有常量 `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0`，**不新建**常量 |
| `frontend-rs/tests/tokens.rs`（或新建 `tests/table_size.rs`） | 新增 **UT-MM-17**（happy + edge，编号 `UT-MM-17` —— `UT-MM-10..16` 已被 `validate_language`/`custom_type`/`import_source` 等占用） |
| `logos/resources/test/core-CR-canvas-test-cases.md`（或同类） | 登记 UT-MM-17 |

### UT 编号策略

- **UT-MM-10..16 已被占用**（grep `UT-MM-` 确认）：
  - UT-MM-12: `validate_language`（`editor_panels.rs:7484`）
  - UT-MM-13: `add/remove_custom_type`（`editor_panels.rs:8184`）
  - UT-MM-14: `import_source`（推测，需 grep 确认）
  - UT-MM-10/11/15/16: 既有其它
- **新用例取 `UT-MM-17`**（下一空闲编号）

### 验收门槛

1. **UT-MM-17 通过**：
   - happy: `parse_table_height("200") → Ok(200)`
   - happy: `parse_table_height("100") → Ok(100)`
   - **edge: `parse_table_height("0") → Ok(0)`**（**0 = auto，对称 width**）
   - edge: `parse_table_height("abc") → Err(...)`
   - edge: `parse_table_height("") → Err(...)`
   - edge: `parse_table_height("-5") → Err(...)`（负数被拒绝）
2. **`SetTableWidthModal` Apply 闭环测试**：点击 Apply 后，store 中选中 table 的 `width` 字段被更新；Canvas 重新渲染时使用新宽度（手动 e2e 验证或新增 Playwright 用例）
3. **Cargo test 全绿**（frontend-rs + backend + mcp-server）
4. **Playwright 视觉回归**：
   - 一张 8 字段的表，min_height=100 → 实际渲染高度 max(100, 43+35×8=323) = 323
   - 一张 3 字段的表，min_height=300 → 实际渲染高度 max(300, 43+35×3=148) = 300
   - 一张表 width=400 → 实际渲染宽度 400（替换硬编码 230）
   - reference 连线端点不与新宽度/高度重叠（外环 R1 风险点）
5. **`openlogos verify` Gate 3.5 PASS**（回归无破坏）

### 风险点

- **R1**：Canvas 渲染宽高动态化可能让 reference 连线端点计算偏移（`editor_render.rs:935` 处 `(table.x + TABLE_WIDTH, field_anchor_y(...))`） → 需在 Canvas draw 与 reference 重算函数之间加联调测试
- **R2**：模态视觉与现有 `ST-FE-ALIGN-*` 测试视觉断言不能破坏
- **R3**：`Table.width` 默认 `None` 走 `TABLE_WIDTH=230.0` 硬编码 fallback——若实现时把 fallback 常量定义错位置会出现常量重复（提案范围已规定**复用**现有常量）
- **R4**：与未来 `feat-relation-inference` 的端点计算耦合（本提案引入 `width` 后，端点从 `table.x + TABLE_WIDTH` 改为 `table.x + table.width`）—— 本提案同时修复 reference 端点计算（范围扩展明示）
- **R5**：OT 操作需携带 `width` 和 `min_height`（新增字段），需检查 op 应用器对未知字段的处理（推测 silently ignore，实现时验证）

### 替代方案否决理由

- **替代 A：绝对高度**（用户输入数值，表按该高度固定）：字段多时溢出、少时空洞——否决（与 Q4 operator 裁决相反）
- **替代 B：完全自动**（不允许手动调）：等于零工作量不算新功能——否决（operator 显式要求用户可控）
- **替代 C：宽度+高度合并为一个 `size: { w, h }` 对象**：破坏向后兼容——否决（与 Q9 operator 要求的"宽度复用既有 parse_table_width 路径"原则冲突；保持 `width` + `min_height` 平铺两个字段）
- **替代 D：放在 `ux-canvas-batch`（C 案）合并做**：会增加 C 案工作量至 10+ 天，违反"5 案串行"节奏；且表宽高是独立可交付切片——否决（operator 已在推荐顺序中把 A 列为首案）
- **替代 E：只做 `min_height` 不做 `width`**：v1 草案的范围；外环 R2 已批驳——`SetTableWidthModal` Apply 无 on:click 是空壳，必须修复；渲染消费 `TABLE_WIDTH=230.0` 硬编码也必须替换——否决

### 部署影响

- 是否需要部署：**否**（纯前端 WASM；本地开发重新构建即生效）
- 是否涉及数据迁移：**否**（向后兼容，老数据 width/min_height = None 走硬编码 fallback，视觉无突变）
- 是否需要回滚预案：**否**（小切片，回滚 = revert commit）
- 是否需要 smoke：**否**（开发阶段，无独立部署节点）

### UI/UX 变更声明

```yaml
ui_impact: true                # 模态 Apply 闭环 + 新增 min_height 模态（或扩展 size 模态）
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - editor-modal-set-width      # 已有，Apply 闭环
  - editor-modal-set-min-height # 新增（或 size 模态合并实现时定）
  - canvas-table-rendering      # 渲染消费 width/min_height
```

**视觉变化**：
- 现有 SetTableWidthModal 的 Apply 按钮从"假动作"变为真正生效
- Canvas 表宽度从固定 230.0 变为可配置（None 时仍 230.0，老数据视觉一致）
- Canvas 表最小高度可配置（None 时按字段数自动撑高，与现状一致）
- 新增 min_height 模态入口（待 UI 设计定具体入口位置）

### 关联场景

- **S01（编辑并保存图表）**：width/min_height 字段加入 store → 保存链路需携带（已是 JSON blob 全量保存，自动）
- **S05（OT 实时协作）**：width/min_height 字段加入后，OT 操作需包含此字段（实现时验证 op 序列化）
- **S02（分享链接图表）**：老数据 width/min_height = None 走硬编码 fallback，分享出去的图表视觉无突变

### 关联任务清单

见 `05-feat-table-resize-tasks.md`。

## 不在范围（明确排除）

- 不修改 Inspector 任何字段（v1 草案错误地把模态当 Inspector；v2 明确是模态）
- 不修改 reference 连线布局的端点计算逻辑（仅跟随 width 变化；属 R1 风险的伴随修改，不引入新算法）
- 不修改表的固定行高常量 `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0`（属样式优化，归 C 案）
- 不加拖拽 resize 手柄（属 UX 大改动，归 C 案）
- 不改字段顺序拖拽（属需求 6，归 E 案）