# Delta — core-CR-canvas-test-cases.md（修改）

> merge 时按 ADDED 标记合并到 `logos/resources/test/core-CR-canvas-test-cases.md`

> 模块：core | 提案：fix-table-header-width-autofit

## ADDED — fix-table-header-width-autofit 表头行宽度自适应

### UT-CR-HEADER-WIDTH-01 — 表头行宽度需求不受 Tier 上限钳制

- **位置**：`frontend-rs/src/editor_render.rs`（`resolve_table_width_for` / `estimate_content_width_for` 拆分表头行需求与字段行需求）
- **前置**：构造一张表：表名 `asset_kind_catalog`（18 ASCII），表注释 `资产类型字典；客户确认分类后由 seed 脚本初始化`（20 CJK + 5 ASCII，表头行需求 ≈507px > 480），字段 `id BIGINT`（短字段行）
- **步骤**：
  1. auto 宽（`width = None`）下调用 `resolve_table_width_for(table, NameComment, 1.0, 1.0, LodTier::Detail)`
  2. 同表调用 `resolve_table_width_for(table, NameComment, 1.0, 1.0, LodTier::Topology)`
  3. 构造短表头表（表名 `t`、无注释、1 个短字段）同口径调用 Detail
  4. 构造超长注释表（表头行需求 > 720px）同口径调用 Detail
  5. 对步骤 1 的表设置 `width = Some(400)` 后再次调用
- **断言**：
  - 步骤 1 宽度 > `TABLE_WIDTH_MAX`（480）——表头需求突破 Tier 上限
  - 步骤 1 宽度 ≤ `TABLE_WIDTH_MAX_HEADER`（720）
  - 步骤 2 宽度 ≥ 步骤 1 宽度（Topology 下同口径，不小于 Detail 结果）且 > `TABLE_WIDTH_MAX_TOPOLOGY`（640）当表头需求落在 (640, 720]
  - 步骤 3 宽度 = `TABLE_WIDTH`（230）——短表头行为不变（R-LOD-10 既有口径不回退）
  - 步骤 4 宽度 = `TABLE_WIDTH_MAX_HEADER`（720）——绝对上限钳制
  - 步骤 5 宽度 = 400——显式宽度语义不变（不做内容自适应）
  - 字段行需求超上限的表（长字段名 + 长类型行）：宽度仍钳在 Tier 上限（480/640）——字段行钳制结构不变

### ST-CR-HEADER-WIDTH-01 — 字段维度下长中英文注释表头完整显示

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 1 张表（x=100, y=100，auto 宽），表名 `asset_kind_catalog`，表注释 `资产类型字典；客户确认分类后由 seed 脚本初始化`，字段 `id BIGINT`
- **WHEN**：进入房间，保持字段维度（默认），等待画布渲染
- **THEN**：
  1. 表卡渲染宽度 > 480（canvas 探针 / `data-follow-path` 端点几何或 PUT payload 宽度口径读取）
  2. 表头注释文本无 `…` 截断：canvas 文本探针断言注释完整可见（或可见长度 ≥ 注释全长的 95%）
  3. 表名与注释并排不重叠，字段计数仍右对齐可见
  4. 显式拉宽到 400 的对照表宽度保持 400（用户固定宽不受自适应影响）
- **reporter**：`ST-CR-HEADER-WIDTH-01` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-HEADER-WIDTH-01 | 表头行宽度需求不受 Tier 上限钳制 | `editor_render.rs::resolve_table_width_for` |
| ST-CR-HEADER-WIDTH-01 | 字段维度下长中英文注释表头完整显示 | e2e canvas header width autofit |
