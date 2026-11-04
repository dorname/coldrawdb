# Delta — core-S06-test-cases.md（fix-mcp-tool-gaps-issue76）

> 模块：core | 场景：S06 | 关联 issue #76：MCP 工具层缺口（删表 / areas / notes 增量写 / 防误删守护）

## ADDED — UT-MCP-51 — update_table action=delete 删表与引用清理

- **位置**：`mcp-server/tests/c6_write_contract.rs`，mock 后端（GET + PUT 序列）。
- **GIVEN**：图中 t1（含字段 f1/f2）、t2，r1 连接 t1→t2，r2 连接 t2→t1。
- **WHEN / THEN**：
  1. `update_table(action=delete, table_id=t1)` 成功：PUT 载荷 tables 仅剩 t2；references 为空（r1/r2 均触碰 t1 被清理）；响应含 `deleted_table_id="t1"` 与 `deleted_reference_ids=["r1","r2"]`。
  2. `action=delete` 且携带 `name` → VALIDATION_ERROR，零写入（无 PUT）。
  3. `action=delete` 且 `table_id` 不存在 → VALIDATION_ERROR，零写入。
- **reporter**：真实运行后登记 `UT-MCP-51`。

## ADDED — UT-MCP-52 — update_area 三动作（create/update/delete）

- **位置**：同上。
- **WHEN / THEN**：
  1. create：必填 name，缺省 x/y/width/height/color/locked 落默认值；服务端重键 auto-，响应 `area_id` 为实际 ID；PUT 载荷 areas 含新区域全字段。
  2. update：按实际 area_id 改 name+width，未携带属性（x/y/color/locked）保持原值；area_id 不存在 → VALIDATION_ERROR 零写入。
  3. delete：区域从 PUT 载荷消失；携带禁字段（如 name）→ VALIDATION_ERROR 零写入。
- **reporter**：真实运行后登记 `UT-MCP-52`。

## ADDED — UT-MCP-53 — update_note 三动作（create/update/delete）

- **位置**：同上。
- **WHEN / THEN**：
  1. create：必填 content，服务端重键，响应 `note_id` 为实际 ID；PUT 载荷 notes 含新便签。
  2. update：按实际 note_id 改 content+color，x/y 保持原值；note_id 不存在 → VALIDATION_ERROR 零写入。
  3. delete：便签从 PUT 载荷消失；携带禁字段 → VALIDATION_ERROR 零写入。
- **reporter**：真实运行后登记 `UT-MCP-53`。

## ADDED — UT-MCP-54 — update_diagram 防误删守护（allow_mass_delete）

- **位置**：同上（GET base + 校验本地拦截，不发上游写）。
- **WHEN / THEN**（base 3 表 t1/t2/t3）：
  1. desired 缺 t1（缺失 1/3 ≤50% 且 ≤10）→ 正常放行。
  2. desired 仅含 t1（缺失 2/3 >50%）且无 allow_mass_delete → VALIDATION_ERROR，details 含缺失计数，零写入（无 PUT、无 WS）。
  3. 同 2 但 `allow_mass_delete=true` → 放行执行。
- **reporter**：真实运行后登记 `UT-MCP-54`。

## MODIFIED — UT-MCP-50 — 工具 schema 客户端兼容守卫（锚点更新）

断言 4 的工具总数锚点 12 → **14**，且包含 `update_area` / `update_note`；
断言 1 扩展：新工具 update_area/update_note 的 schema 同样整树无
allOf/oneOf/anyOf/if/then/else/not。

## ADDED — ST-MCP-22 — 真实 stdio tools/list 14 工具探针

- **位置**：`mcp-server/tests/`（既有 stdio 探针模式，如 c6 ST-MCP-14 同款子进程 + JSON-RPC）。
- **WHEN / THEN**：release 二进制真实 stdio `tools/list` 返回 14 个工具，含
  `update_table`（action enum 含 delete）、`update_area`、`update_note`；
  新工具 schema 抽样断言平铺结构（顶层 type/properties/required）。
- **reporter**：真实运行后登记 `ST-MCP-22`。

## ADDED — 用例登记（OpenLogos verify 解析用，fix-mcp-tool-gaps-issue76）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-51 | UT | update_table delete 删表、触碰 references 清理、禁字段与不存在零写入 |
| UT-MCP-52 | UT | update_area create/update/delete 全动作与校验 |
| UT-MCP-53 | UT | update_note create/update/delete 全动作与校验 |
| UT-MCP-54 | UT | update_diagram 缺表守护三态（放行/拦截/确认放行） |
| ST-MCP-22 | ST | 真实 stdio tools/list 14 工具与新工具 schema 平铺探针 |

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09、UT-MCP-50 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、UT-MCP-50、UT-MCP-51～53、ST-MCP-03/14/18 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、UT-MCP-29、UT-MCP-45～49、UT-MCP-54、ST-MCP-04/05/19～21 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、UT-MCP-47、ST-MCP-06～09/20、ST-MCP-22 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27、UT-MCP-44～48、ST-MCP-19/20 |
