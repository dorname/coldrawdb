# Delta — core-PE-design-system-test-cases.md（fix-31-related-line-emphasis / #31 提亮）

> 模块：core | 提案：fix-31-related-line-emphasis
> 目标：UT-PE-HL-01 / ST-PE-10 断言口径随 §4.4 提亮合同修订——相关连线强制选中色族 + 发光光晕 + 端点同色。不新增用例 ID。

## MODIFIED — UT-PE-HL-01 — 选中高亮参数纯函数（#31；fix-31-related-line-emphasis 提亮）

- **位置**：`frontend-rs/src/editor_render.rs`（高亮态解析纯函数，如 `relation_render_width(base, related)` / `table_render_alpha(...)` + `draw_relation` 相关态分支源码锚点）
- **断言**：
  1. 选中一张表后：相关关系线线宽 = 默认 1.5×（`RELATED_RELATION_WIDTH_FACTOR`）且 alpha = 1；**主色强制 `palette.selected`、光晕 `palette.selected_soft`（8px，选中关系自身 10px）**；非相关关系线 alpha ≤ 0.25；非相关表 alpha ≤ 0.5；邻接表 alpha = 1
  2. 选中一条关系后：该关系为选中态（3.5px 主线 + 10px 光晕），其两端表为相关态；共享端点表的其余关系按相关线渲染；其余按非相关处理
  3. 无选中：全部恢复默认线宽、关系解析色（§4.1/§4.2）与 alpha = 1
  4. 只读模式下同样的输入产生同样的视觉参数（只读不改变高亮反馈）
  5. crow's foot 端点记号与主线同色（选中/相关态不得回落基线色 `stroke`）——`draw_relation` 源码锚点

## MODIFIED — ST-PE-10 — e2e：选中高亮与注释可读性目视锚点（#31 / #32；fix-31-related-line-emphasis 提亮）

> 改号注记：原编号 ST-PE-08 与既有「ST-PE-08 — Monaco 加载」（E4 CodeView）撞号，按场景编号全局唯一规则改为 ST-PE-10（ST-PE-09 已由 #33 占用）。

- **位置**：`frontend-rs` e2e（`scripts/test-spec-parity-d.mjs`）
- **GIVEN**：画布含 ≥3 张表（表头色分别蓝 / 绿 / 紫）、≥3 条关系、表与字段含中文注释；注释显示模式 = 英文名+注释
- **WHEN**：选中一张有 ≥3 条关系的表；随后切换亮 / 暗主题各观察一次
- **THEN**：
  1. 相关连线以**选中色族**亮显并带发光光晕且更粗（`__cdb_hl_probe` 暴露 `related_emphasis` 字段为 true + `rel_width_scale_max` ≥ 1.5），非相关表/线退到背景层（`table_alpha_min` ≤ 0.5）；取消选中后探针复位
  2. 三种表头色 × 两主题下表头注释均可读（无近白字压浅底 / 深色字压深渐变）
  3. 亮 / 暗主题截图锚点存在可分辨差异且相关线区域命中选中色族像素
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## MODIFIED — ID 登记表（OpenLogos ledger，更新行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PE-HL-01 | UT | 选中高亮参数纯函数（相关线 1.5× + 选中色族 + 发光 halo / 非相关线 ≤0.25 / 非相关表 ≤0.5 / 端点同色） |
| ST-PE-10 | ST | 选中高亮与注释可读性 e2e 目视锚点（原 ST-PE-08 改号；fix-31 提亮：相关线选中色族探针断言） |
