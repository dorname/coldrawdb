# Delta: core-09-core-components.md

> 提案：fix-select-bg-diagram-delete-and-log-format

## ADDED — §13 表单下拉（追加第 5 条）

5. **禁止 `background` 简写覆盖**（fix-select-bg-diagram-delete-and-log-format）：任何同时命中 select 的复合选择器（如 `.cdb-create-room-modal .cdb-input`）**不得**使用 `background:` 简写——简写会把 `background-image`/`background-repeat` 重置，压过 `.cdb-select` 的 chevron 单箭头（优先级 0,2,0 > 0,1,0），暗色规则重设 image 后 repeat 失控 → 箭头平铺成满框 `˅˅˅`。必须改用 `background-color:` 单独声明。锚点：UT-PC-23。
