# Delta — core-CR-canvas-test-cases.md（修改）

> 模块：core | 提案：fix-table-auto-width-include-comment
> 关联 issue：#25

## ADDED — 合并自 fix-table-auto-width-include-comment（2026-09-21）

### 用例登记（verify 表行解析）

| ID | 描述 |
|---|---|
| UT-CR-WIDTH-03 | NameComment 中文注释并排撑宽（#25） |

## ADDED — UT-CR-WIDTH-03 — NameComment 中文注释并排撑宽（#25）

- **位置**：`frontend-rs/src/editor_render.rs`（`estimate_content_width` / `resolve_table_width`）
- **GIVEN**：
  1. 表 `width=None`，短表名（如 `t`），字段名 `id`，类型 `VARCHAR(32)`，字段注释「主键」，显示模式 `NameComment`
  2. 对照：同一表去掉字段注释（其余不变）
- **断言**：
  1. 有注释时 `estimate_content_width` **严格大于** 无注释对照（未触达 `TABLE_WIDTH_MAX` 前；若两者均被 230 下界夹紧则改用更长注释或更长类型使差值可见）
  2. 有注释估算 ≥ `measure_text_approx(名) + measure_text_approx(注释) + measure_text_approx(类型) + 间距常量 + PAD` 的近似下界（允许实现常量微调，但不得退回「注释与名+类型取 max」）
  3. `CommentDisplay::Name` 下有/无字段注释估算相等（无 secondary）
- **报告**：结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，用例 ID `UT-CR-WIDTH-03`）

> 附录追加：UT-CR-WIDTH-03。
