# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/api/mcp-tools.yaml` — 新增 `update_dictionary` 工具（create/update/delete 三 action）+ `update_field` 增补 `dict_code` 参数
- [x] D2 产出 delta `deltas/prd/3-technical-plan/2-scenario-implementation/core-S07-data-dictionary.md` — §10 S06 口径改写为「MCP 正式支持字典管理」+ 工具语义段落
- [x] D3 产出 delta `deltas/test/core-S06-test-cases.md` — 新增 UT-MCP-33（create+编码唯一校验）、UT-MCP-34（update 改码同步引用 + delete 级联置空）、UT-MCP-35（update_field dict_code 绑定/解绑/悬空拒绝）
- [x] D4 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [x] C1 `mcp-server/src/service.rs` 实现 `update_dictionary`（create/update/delete + 编码唯一校验 + 改码同步引用 + 删除级联置空）与 `update_field` 的 `dict_code` 分支；列出本批覆盖 UT 用例 ID（UT-MCP-33/34/35）
- [x] C2 编写 UT-MCP-33/34/35 测试并同批写入 OpenLogos reporter（`mcp-server/tests/` 或既有测试通路）

## [follow-up] 收尾

- [x] F1 verify PASS 后 `openlogos archive mcp-dictionary-tools`（/goal 授权），随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal「需要AI 通过 MCP 正式管理字典」全程授权）
- [x] H2 delta 完成后 `openlogos merge mcp-dictionary-tools`（/goal 授权）
- [x] H3 merge 后自动提交规格文档并按合并规格实现代码，完成后自动提交
- [x] H4 实现完成后 `openlogos verify`（/goal 授权；nice -n 10 + 测试线程限制）
- [x] H5 verify PASS 后 `openlogos archive`（/goal 授权）
- [x] H6 归档提交完成后 `git push`（/goal 授权）
