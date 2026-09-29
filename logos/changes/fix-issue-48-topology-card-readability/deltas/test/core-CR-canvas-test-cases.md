## ADDED — fix-issue-48-topology-card-readability / #48 表维度卡宽上限

### UT-CR-TOPO-WIDTH-01 — 表维度卡宽上限高于字段维度

- **位置**：`frontend-rs/src/editor_render.rs`（`resolve_table_width_for` / `estimate_content_width_for`）
- **前置**：构造一张表，表名 ≥ 10 ASCII，表注释 ≥ 8 CJK，字段名与类型正常
- **步骤**：
  1. 调用 `resolve_table_width_for(table, NameComment, 1.0, 1.0, LodTier::Detail)`
  2. 调用 `resolve_table_width_for(table, NameComment, 1.0, 1.0, LodTier::Topology)`
- **断言**：
  - 字段维度宽度 ≤ `TABLE_WIDTH_MAX`（480）
  - 表维度宽度 > 字段维度宽度（上限放宽后长文本可撑宽）
  - 表维度宽度 ≤ `TABLE_WIDTH_MAX_TOPOLOGY`（640）
  - 空表/短名表在两种维度下均回落至 `TABLE_WIDTH`（230），宽度一致

### ST-CR-TOPO-WIDTH-01 — 表维度下长中文注释表卡完整可读

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 1 张表（x=100, y=100），表名 `asset_object_registry`，表注释 `统一资产主表用于资产管理系统`，字段 `id INT`
- **WHEN**：进入房间后切换为表维度（点击 FloatingControls「维度：表」/快捷键 V），等待画布重绘
- **THEN**：
  1. 表维度卡宽大于字段维度下同一表的卡宽（data-follow-path 端点或 canvas 探针读取宽度）
  2. 表名或注释未出现大量 `…` 截断：canvas 探针或文本像素断言主文案可见长度 ≥ 表名/注释长度的 80%
  3. 切换回字段维度后宽度回落、字段行出现，无回归
- **reporter**：`ST-CR-TOPO-WIDTH-01` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-TOPO-WIDTH-01 | 表维度卡宽上限高于字段维度 | `editor_render.rs::resolve_table_width_for` |
| ST-CR-TOPO-WIDTH-01 | 表维度下长中文注释表卡完整可读 | e2e canvas topology width |
