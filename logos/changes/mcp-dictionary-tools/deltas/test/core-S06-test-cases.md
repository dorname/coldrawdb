# Delta — core-S06-test-cases.md（mcp-dictionary-tools）

> 模块：core | 提案：mcp-dictionary-tools
> 目标：新增 `update_dictionary` 工具与 `update_field.dict_code` 的用例；UT-MCP-20 工具总数 11 → 12。

## MODIFIED — UT 用例（UT-MCP-20 行改写）

| UT-MCP-20 | tools/list 工具总数 | 恰好 12 个工具；新增 5 个工具（update_table/update_field/update_reference/layout_diagram/update_dictionary）name/title/description/inputSchema/outputSchema 稳定 |

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-33 | `update_dictionary` create + 编码唯一校验 | create 缺 name/code → `VALIDATION_ERROR` 零请求；code 与既有字典冲突 → `VALIDATION_ERROR` 零请求（对齐 S07 EX-7.1）；合法 create（含 items，缺 id 项自动补 id）→ PUT body `dictionaries` 追加一条且其余图元不变，瘦响应含 `{id, revision, dict_id, fields_cleared:0}` |
| UT-MCP-34 | `update_dictionary` update 改码同步引用 + delete 级联置空 | update 改 code → 所有 `dict_code` 指向旧编码的字段同步改写为新编码（无悬空）；改 code 与另一字典冲突 → `VALIDATION_ERROR`；delete → 字典移除且引用字段 `dict_code` 置空，瘦响应 `fields_cleared` = 实际置空字段数（对齐 S07 §5）；dict_id 不存在 → `VALIDATION_ERROR` 零请求 |
| UT-MCP-35 | `update_field` dict_code 绑定/解绑/悬空拒绝 | 绑定已存在字典编码 → PUT body 该字段 `dict_code` 更新；`dict_code=""` → 解绑置空；绑定不存在编码 → `VALIDATION_ERROR` 且 mock HTTP 断言零请求发出（杜绝 S07 EX-7.4 悬空绑定） |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-33 | UT | update_dictionary create + 编码唯一校验 + 瘦响应结构 |
| UT-MCP-34 | UT | update_dictionary update 改码同步引用 + delete 级联置空 fields_cleared |
| UT-MCP-35 | UT | update_field dict_code 绑定/解绑/悬空拒绝零请求 |
