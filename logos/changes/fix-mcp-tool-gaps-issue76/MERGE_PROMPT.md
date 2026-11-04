# 合并指令

## 变更提案
- 提案名称：fix-mcp-tool-gaps-issue76
- 提案目录：logos/changes/fix-mcp-tool-gaps-issue76/

## 提案内容

# 变更提案：fix-mcp-tool-gaps-issue76

> module: core | created: 2026-11-04 | 关联 issue: #76

## 变更原因

GitHub issue #76：MCP 工具层缺口导致大图维护被阻断。对 `dspm-ent-full-er`
（58 表 / 80 关系，全文档 PUT 体 226,572 字节）做收口维护时，纯 MCP 路径无法完成
两类常见操作：

1. **删表**：`update_table` 只有 `action: create | update`，没有 delete。删表唯一
   入口是 `update_diagram` 全文档 PUT，而它被 #70 的客户端 ~64KB 参数截断拦死
   （227KB ≈ 上限 3.5 倍，截断成非法 JSON 干净失败）。
2. **areas / notes 增量写**：无对应细粒度工具，同样只能全文档 PUT。

瓶颈不在服务端：`payload_limit.rs` 软限 256KiB，REST 直写同构请求体实测成功——
纯工具层缺口。「只传增量」也不可行：房间收编路径 `update_diagram` →
`write_room_diagram` → `collab/diff.rs:102-107` 的 diff 语义是**缺失 section 即
删除**，传精简文档会把其余表全部删掉。

## 变更类型

接口级变更（MCP 工具契约扩展：既有工具增加 delete action、新增两个细粒度工具、
全文档写路径增加防误删守护；既有 create/update 语义与错误码不变）

## 变更范围

- 影响的需求文档：无（S06 需求的 12 工具清单为存量描述；本提案扩展工具面并在
  契约与测试用例中登记，工具总数 12 → 14）
- 影响的功能规格：`logos/resources/api/mcp-tools.yaml` —
  ① `update_table` inputSchema 的 action 扩为 `create|update|delete`（delete 禁
  携带 name/fields 等字段，description 写明顺带清理触碰 references）；
  ② 新增 `update_area` / `update_note` 工具（create/update/delete，schema 平铺
  遵守客户端兼容约束）；③ `update_diagram` 新增 `allow_mass_delete` 守护参数与
  「缺失 section = 删除」语义说明；④ PAYLOAD_TOO_LARGE 分片建议补充增量工具提示
- 影响的业务场景：S06（MCP stdio 服务；新增两类增量写时序，复用既有
  GET→改→PUT/分批模式，不新增编排场景）
- 影响的 API：MCP 工具 `update_table`、`update_diagram`（参数扩展 + 守护），
  新增 MCP 工具 `update_area`、`update_note`；upstream REST/WS 端点零变化
- 影响的 DB 表：无
- 影响的编排测试：无（MCP 无编排测试）
- 影响的测试用例：`logos/resources/test/core-S06-test-cases.md` 新增
  UT-MCP-51～54 与 ST-MCP-22；MODIFIED UT-MCP-50（工具总数锚点 12 → 14，
  新增 update_area/update_note schema 平铺断言）

## 部署影响

- 是否需要部署：否
- 部署原因：MCP 为本地 stdio 服务，无独立部署环境；交付含 release 二进制重建
  （契约在编译期经 `include_str!` 内嵌），客户端重连即生效
- 影响环境：无（本地）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **`update_table` 增加 `action: delete`**：按 `table_id` 删表；服务端
   GET→改→PUT 顺带移除触碰该表的 references（start_table_id/end_table_id 命中），
   维持无悬空；返回 slim 响应 + `deleted_table_id` + `deleted_reference_ids`。
   delete 携带 name/fields/comment/x/y/color/locked 任一即 VALIDATION_ERROR
   零写入；目标表不存在 VALIDATION_ERROR。
2. **新增 `update_area` / `update_note`**：与 `update_table` 同模式
   （GET→改→PUT），action=create|update|delete。
   - update_area：AreaDto 字段 id/x/y/width/height/color/name/locked；
     create 必填 name，服务端重键 auto-；update/delete 按 area_id。
   - update_note：NoteDto 字段 id/x/y/content/color；create 必填 content，
     服务端重键 auto-；update/delete 按 note_id。
   - 单次调用小载荷，不受全文档体积约束（绕开 #70 客户端截断）。
3. **`update_diagram` 防误删守护**：写前本地 GET base 比较——desired 缺失表数
   > 10 或缺失比例 > 50%（其一即触发）时，必须显式携带 `allow_mass_delete: true`
   才执行，否则 VALIDATION_ERROR（details 列缺失表数与样例）零写入；工具
   description 写明「缺失 section = 删除」语义。个人图直写与房间收编路径
   （write_room_diagram）统一守护。
4. **PAYLOAD_TOO_LARGE 建议文案**：suggestion 补充「删表用
   update_table(action=delete)；areas/notes 用 update_area/update_note 增量写」。
5. **测试**：UT-MCP-51（delete 删含关系表 + 禁字段 + 不存在零写入）、
   UT-MCP-52/53（area/note 三动作全断言）、UT-MCP-54（守护三态：不触发/拦截/
   放行）、UT-MCP-50 扩展（14 工具 + 新工具 schema 平铺）、ST-MCP-22（真实
   stdio tools/list 14 工具探针）。release 二进制重建 + stdio 探针复核。


## 需要合并的 Delta 文件

### 1. deltas/api/mcp-tools.yaml

- Delta 文件：`logos/changes/fix-mcp-tool-gaps-issue76/deltas/api/mcp-tools.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-mcp-tool-gaps-issue76/deltas/test/core-S06-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(fix-mcp-tool-gaps-issue76): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-mcp-tool-gaps-issue76`。
