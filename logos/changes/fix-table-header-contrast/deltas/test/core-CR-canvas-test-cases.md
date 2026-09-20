# Delta — core-CR-canvas-test-cases.md（修改）

> 模块：core | 提案：fix-table-header-contrast
> 关联 issue：#24

## ADDED — 合并自 fix-table-header-contrast（2026-09-20）

### 用例登记（verify 表行解析）

| ID | 描述 |
|---|---|
| UT-CR-COLOR-03 | 表头前景相对有效背景亮度自适应（R-COLOR-04） |

## ADDED — UT-CR-COLOR-03 — 表头对比度自适应（#24）

- **位置**：`frontend-rs/src/editor_render.rs`（表头有效背景合成 + 前景选择纯函数；`draw_table_body` 表头文字消费）
- **步骤 / 断言**：
  1. 浅色实色表头（如 `#e8eef0` / `#ffffff`）相对暗色 `table_bg` 合成后 → 强/次前景取深色对（对齐 `PALETTE_LIGHT.text_strong` / `text_muted`）
  2. 深色实色表头（如 `#175e7a` / `#142c34`）→ 强/次前景取浅色对（对齐 `PALETTE_DARK` 对应字段）
  3. 空 `table.color` + 暗主题默认 `header_tint` 半透明合成后 → 仍选浅色前景（与现状暗主题可读一致）
  4. 空 `table.color` + 亮主题默认 tint 合成后 → 仍选深色前景
  5. 表色接近主题字色（如暗主题下 `#f2fdfe`）→ 不得继续用浅色字；须切到深色前景
  6. 渲染锚点：`draw_table_body` 表头 `fill_text` 前消费自适应前景，不得写死 `palette.text_strong` / `text_muted` 作为表头唯一取色源
- **报告**：结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，用例 ID `UT-CR-COLOR-03`）

> 附录追加：UT-CR-COLOR-03。
