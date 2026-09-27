# Delta — core-PE-design-system-test-cases.md（fix-open-issues-26-33 / #31 + #32 + #33）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：
> - #31/#32：选中高亮强化与注释对比度的渲染层用例（纯函数 + e2e 目视锚点）。
> - #33：视觉体系统一的一致性用例（图标族/token 锚点 + e2e 目视）。

## ADDED — UT-PE-HL-01 — 选中高亮参数纯函数（#31）

- **位置**：`frontend-rs/src/editor_render.rs`（高亮态解析纯函数，如 `relation_render_style(is_related)` / `table_render_alpha(is_related)`）
- **断言**：
  1. 选中一张表后：相关关系线线宽 ≥ 默认 1.5× 且 alpha = 1；非相关关系线 alpha ≤ 0.25；非相关表 alpha ≤ 0.5；邻接表 alpha = 1
  2. 选中一条关系后：该关系及其两端表为相关态；其余按非相关处理
  3. 无选中：全部恢复默认线宽与 alpha = 1
  4. 只读模式下同样的输入产生同样的视觉参数（只读不改变高亮反馈）

## ADDED — UT-PE-CMT-01 — 注释对比度前景计算纯函数（#32）

- **位置**：`frontend-rs/src/editor_render.rs`（注释前景/衬底决策纯函数，如 `comment_foreground(header_segment_bg)` / `needs_comment_chip(contrast)`）
- **断言**：
  1. 注释区有效背景（渐变段色）亮度高于阈值 → 选深色前景对；低于阈值 → 浅色前景对（与 R-COLOR-04 同阈值口径）
  2. 前景对对比度仍 < 4.5:1 → `needs_comment_chip` 返回 true（启用衬底/弱化渐变兜底）；≥ 4.5:1 → false
  3. 蓝 / 绿 / 紫三种预设表头色 × 亮 / 暗主题共 6 组输入，输出前景均满足 ≥ 4.5:1（含兜底生效情形）
  4. 空注释输入不产生任何注释渲染参数（空注释不占位不回归）

## ADDED — ST-PE-08 — e2e：选中高亮与注释可读性目视锚点（#31 / #32）

- **位置**：`frontend-rs` e2e（独立 spec 或并入 cr-comment-color.spec）
- **GIVEN**：画布含 ≥3 张表（表头色分别蓝 / 绿 / 紫）、≥3 条关系、表与字段含中文注释；注释显示模式 = 英文名+注释
- **WHEN**：选中一张有 ≥3 条关系的表；随后切换亮 / 暗主题各观察一次
- **THEN**：
  1. 相关连线明显更粗更亮，非相关表/线退到背景层（截图锚点 + 关键 alpha/线宽断言经 `data-testid` 或 canvas 参数探针）
  2. 三种表头色 × 两主题下表头注释均可读（无近白字压浅底 / 深色字压深渐变）
  3. 取消选中后全图恢复正常
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## ADDED — UT-PE-VIS-01 — 语义图标统一族锚点（#33）

- **位置**：`frontend-rs/src/editor_render.rs` / 图标组件 + `include_str!` 锚点
- **断言**：
  1. PK / FK / NOT NULL / UNIQUE 徽章渲染共用统一尺寸（12px）与描边（1.5px）常量（同源引用，无散值）
  2. 徽章色值仅取自设计 token 语义色阶（源码锚点无裸 hex 散值）
  3. ToolRail 图标统一 stroke（1.5px）与尺寸（20px / IconBox 24px）；激活态规则存在（主色描边 + 浅底）
  4. 字块角标渲染路径标记 deprecated 或已移除（不再与图标族混用）

## ADDED — ST-PE-09 — e2e：视觉体系统一目视锚点（#33）

- **位置**：`frontend-rs` e2e（独立 spec）
- **GIVEN**：画布含 ≥2 张表（含 PK / FK / NOT NULL / UNIQUE 字段各一）、≥1 条关系；亮 / 暗主题
- **WHEN**：观察表卡、徽章、关系端点、ToolRail；切换主题再观察
- **THEN**：
  1. 表卡圆角 / 投影 / 表头渐变一致（截图锚点 + 关键 CSS/常量断言）
  2. 徽章图标族风格统一（同为线条图标，无字块角标混搭）；关系端点为统一 crow's foot 族
  3. ToolRail 图标描边/尺寸/激活态一致
  4. 主题切换后无风格漂移
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## MODIFIED — ID 登记表（OpenLogos ledger，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PE-HL-01 | UT | 选中高亮参数纯函数（相关线 ≥1.5× / 非相关线 ≤0.25 / 非相关表 ≤0.5） |
| UT-PE-CMT-01 | UT | 注释对比度前景计算（分区背景 + 4.5:1 + 衬底兜底） |
| ST-PE-08 | ST | 选中高亮与注释可读性 e2e 目视锚点 |
| UT-PE-VIS-01 | UT | 语义图标统一族锚点（尺寸/描边/色板/激活态） |
| ST-PE-09 | ST | 视觉体系统一 e2e 目视锚点 |
