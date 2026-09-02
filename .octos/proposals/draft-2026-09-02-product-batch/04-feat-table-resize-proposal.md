# 变更提案：feat-table-resize

> module: core | created: 2026-09-02
> 状态：**草案**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 父批次：[产品优化批次-2026-09-02](./01-current-state.md) 切片 2 / 首案
> 裁决依据：黑板条目 6 operator 批注（Q4=最小高度语义；Q9=完整模板）

## 变更原因

**线上 Bug + UX 缺口**：当前 `editor_panels.rs:7473` `parse_table_width` 已就绪，Inspector 也已暴露宽度输入（UT-MM-11 覆盖），但**没有最小高度调整**——当一张表的字段数 > 5 时，固定默认高度导致字段被截断或 Canvas 渲染溢出；用户也无法控制"紧凑 vs 宽松"的视觉密度。

**operator 裁决（Q4）**：采用**最小高度语义**——用户设最小值，字段多时自动撑高。理由：相对高度更稳（不会因字段数变化而崩布局），同时给用户视觉密度控制权。

**证据**：
- `frontend-rs/src/editor_panels.rs:7473` `parse_table_width(input: &str) -> Result<u32, String>` —— 宽度的纯函数已就绪
- `frontend-rs/src/editor_panels.rs:8136` Inspector 宽度输入 UI 已存在
- `frontend-rs/src/editor_panels.rs:8779` `test_parse_table_width_happy_ut_mm_11` —— UT 模板
- `grep parse_table_height` **无命中** —— 高度纯函数不存在
- 当前表渲染逻辑（Canvas draw）推测用字段数 × 固定行高（需实现时复核）

## 变更类型

代码级修复 + UX 增强（轻度）

## 变更范围

### Why

让用户控制表的**最小高度**，并通过新解析函数 `parse_table_height` 复用宽度路径的纯函数 + UT 模式。

### What

1. 新增 `parse_table_height(input: &str) -> Result<u32, String>`（对称 `parse_table_width`）
2. 表 record 增加可选字段 `min_height: Option<u32>`（向后兼容；缺省 = 默认最小高度常量）
3. Inspector 加"最小高度"输入（对称宽度的 UI 模式）
4. Canvas 渲染：表实际渲染高度 = `max(min_height, fields.len() × row_height)`
5. 新增 UT-MM-12 覆盖 `parse_table_height` happy path + 边界（0/空/非法）
6. 表 record 的 `width` 字段已有 → 不动；本次只加 `min_height`

### 数据契约变更

```rust
// frontend-rs/src/editor_core.rs 或对应 store 类型
pub struct Table {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub fields: Vec<Field>,
    pub width: Option<u32>,       // 既有
    pub min_height: Option<u32>,  // ← 新增（Option 向后兼容）
    // ... 其它既有字段不动
}
```

**向后兼容**：
- 老 JSON 没 `min_height` 字段 → serde 反序列化为 `None` → 渲染走默认最小高度（与现状一致）
- 老数据无需 migration
- DB schema 不变（diagram 整体存 JSON blob，前端字段加 DB 不变）

### 文件影响

| 文件 | 改动 |
|---|---|
| `frontend-rs/src/editor_panels.rs` | 新增 `parse_table_height`；Inspector 加 min_height 输入；Canvas draw 用 `max(min_height, fields×row_height)` |
| `frontend-rs/src/editor_core.rs` | `Table` struct 加 `min_height: Option<u32>` |
| `frontend-rs/tests/tokens.rs`（或新建 `tests/table_height.rs`） | 新增 UT-MM-12 happy + edge |
| `logos/resources/test/core-CR-canvas-test-cases.md`（或同类） | 登记 UT-MM-12（与 UT-MM-11 同格式）|

### 验收门槛

1. **UT-MM-12 通过**：
   - happy: `parse_table_height("200") → Ok(200)`
   - edge: `parse_table_height("0") → Err(...)`（0 应被拒绝，与 width 一致）
   - edge: `parse_table_height("abc") → Err(...)`
   - edge: `parse_table_height("") → Err(...)`
2. **Cargo test 全绿**（frontend-rs + backend + mcp-server）
3. **Playwright 视觉回归**：
   - 一张 8 字段的表，min_height=100，应自动撑高至 ~200（8 × 25px row_height）
   - 一张 3 字段的表，min_height=300，应保持 300（用户最小值生效）
   - reference 连线不与新高度重叠
4. **`openlogos verify` Gate 3.5 PASS**（回归无破坏）

### 风险点

- **R1**：Canvas 渲染高度动态化可能让 reference 连线端点计算偏移 → 需在 Canvas draw 与 reference 重算函数之间加联调测试
- **R2**：Inspector UI 加输入框需小心样式不破坏现有 ST-FE-ALIGN-* 测试视觉断言
- **R3**：老数据 `min_height = None` 走默认值——默认值常量选错会导致视觉突变 → 选与现状一致的行高 × 字段数（动态撑高的现状）
- **R4**：与未来 `feat-relation-inference` 的端点计算可能耦合——本提案不引入 reference 重算逻辑

### 替代方案否决理由

- **替代 A：绝对高度**（用户输入数值，表按该高度固定）：字段多时溢出、少时空洞——否决（与 Q4 operator 裁决相反）
- **替代 B：完全自动**（不允许手动调）：等于零工作量不算新功能——否决（operator 显式要求用户可控）
- **替代 C：宽度+高度合并为一个 `size: { w, h }` 对象**：破坏向后兼容——否决（与 Q9 operator 要求的"宽度复用既有 parse_table_width 路径"原则冲突）
- **替代 D：放在 ux-canvas-batch（C 案）合并做**：会增加 C 案工作量至 10+ 天，违反"5 案串行"节奏；且表宽高是独立可交付切片——否决（operator 已在推荐顺序中把 A 列为首案）

### 部署影响

- 是否需要部署：**否**（纯前端 WASM；本地开发重新构建即生效）
- 是否涉及数据迁移：**否**（向后兼容，老数据 `min_height = None` 走默认）
- 是否需要回滚预案：**否**（小切片，回滚 = revert commit）
- 是否需要 smoke：**否**（开发阶段，无独立部署节点）

### UI/UX 变更声明

```yaml
ui_impact: true                # Inspector 新增"最小高度"输入框
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - editor-inspector            # 右侧 Inspector 面板的"表"分类
```

**视觉变化**：
- 表的渲染高度**可能变大**（老数据无 `min_height` 时与现状一致；有 `min_height` 时按用户输入）
- Inspector 加一个数字输入框（"最小高度(px)"）
- 其余 UI 不动

### 关联场景

- **S01（编辑并保存图表）**：高度是渲染细节，不影响保存链路（数据已存 JSON）
- **S05（OT 实时协作）**：min_height 字段加入后，OT 操作需包含此字段；需检查现有 op 应用器是否对未声明字段 silently 忽略（推测是，但实现时需验证）

### 关联任务清单

见 `05-feat-table-resize-tasks.md`。

## 不在范围（明确排除）

- 不修改 reference 连线布局（属 R1 风险，本提案仅记录不修复）
- 不修改表的固定行高常量（属样式优化，归 C 案）
- 不加拖拽 resize 手柄（属 UX 大改动，归 C 案）
- 不改字段顺序拖拽（属需求 6，归 E 案）