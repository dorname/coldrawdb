# 变更提案：MCP 数据字典管理工具

> module: core | created: 2026-09-28

## 变更原因

S06 MCP stdio 服务当前 11 个工具均不感知数据字典：`get_diagram` 只能被动读出 `dictionaries`，`update_field` 不透传 `dict_code`，AI 客户端无法正式创建/修改/删除字典或绑定字段（只能靠 `update_diagram` 全量裸写，无校验、无引导）。用户目标：「AI 通过 MCP 正式管理字典」。

## 变更类型

接口级变更（MCP 工具契约扩展；无新增 HTTP 端点，复用 GET+PUT 快照通路）

## 变更范围

- 影响的需求文档：无（S07 需求不变，仅扩展 S06 触达面）
- 影响的功能规格：无（纯前端交互不变）
- 影响的业务场景：`core-S07-data-dictionary.md` §10（"S06：本变更不扩展 MCP 工具"口径废止，改为正式支持）；`core-S06` MCP 场景工具清单
- 影响的 API：`logos/resources/api/mcp-tools.yaml`（新增 `update_dictionary` 工具；`update_field` 增补 `dict_code` 参数）
- 影响的 DB 表：无（dictionaries 为 diagram JSON 快照内字段，0007 migration 已覆盖）
- 影响的编排测试：无（编排测试针对 HTTP API，本变更不增 HTTP 端点）
- 影响的 smoke 测试：无
- 影响的测试文档：`logos/resources/test/core-S06-test-cases.md`（新增 UT-MCP-33～35）

## 部署影响

- 是否需要部署：否
- 部署原因：MCP stdio 服务为客户端本地适配器进程，随客户端启动加载新版本；后端 actix-web 与 collab-server 零变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

在 MCP 工具契约中新增 `update_dictionary` 工具（action: create/update/delete），沿用 `update_table`/`update_field` 的「GET 全量读取 → 本地修改 → PUT 全量写回（expected_revision 乐观锁）」分片写入模式：

1. **create**：追加字典 `{id, name, code, comment, items[]}`，本地校验编码唯一性（冲突 → VALIDATION 错误，不发 HTTP），对齐 S07 EX-7.1 口径
2. **update**：按 `dict_id` 定位，就地改写 name/code/comment/items（items 整体替换），改 code 时校验唯一性；改 code 时同步改写所有引用字段的 `dict_code`，避免产生悬空绑定（S07 软引用按编码）
3. **delete**：按 `dict_id` 删除字典，并级联将所有 `dict_code` 指向该字典编码的字段置空，对齐 S07 §5 级联置空语义

同时 `update_field` 增补 `dict_code` 参数（空串 = 解除绑定；非空时本地校验目标字典编码存在，否则 VALIDATION 拒绝，从源头杜绝 EX-7.4 悬空绑定）。
