# 变更提案：fix-issues-42-44-mcp-and-relation-hit（MCP 写入诊断 + 关系线命中/悬浮）

> module: core | created: 2026-09-28

## 变更原因

GitHub 三个 open issue：

- **#42 [BUG]** MCP 无法写入/覆盖已有图表，只能新建（`update_diagram` 报「上游返回非 JSON 响应」）——MCP HTTP 客户端吞掉非 JSON 响应的全部诊断信息（status/Content-Type/body），且 409 `USE_OP_CHANNEL` 识别依赖 body 为特定 JSON，body 非 JSON 时房间图 WS op 通道回退不触发，用户被迫只能 `import_schema` 新建
- **#43 [Enhancement]** 悬浮关系线时展示两端 `表名.字段名`——画布目前无任何 hover 基础设施
- **#44 [BUG]** 关系线密集时点击选中错乱——`hit_test_reference_tier` 双根因：①按 refs 数组序先命中即返回（非最近距离优先）；②命中几何恒为贝塞尔，忽略 `line_type`（orthogonal 折线绘制与命中不同源）

## 变更类型

设计级变更（#43 新增交互、#44 命中合同变更、#42 MCP 错误合同与 tool 描述变更；均涉及规格口径更新）

## 变更范围

- 影响的需求文档：无（issue 即需求来源，验收条件进功能规格）
- 影响的功能规格：
  - `core-01b-relationship.md`（新增关系线命中合同「线型同源 + 最近距离优先」与悬浮 tooltip 交互，#43/#44）
  - `core-S06-mcp-service-design.md`（update_diagram 错误可诊断合同、USE_OP_CHANNEL 鲁棒识别、import_schema「仅新建」文案，#42）
- 影响的业务场景：S01（画布交互域，时序图不变）；S06（MCP 服务，时序图不变——仅错误路径合同收紧）
- 影响的部署方案：`core-01-deployment-plan.md`（前端镜像随 compose 重建部署注记；mcp-server 为 stdio 本机进程，不进 compose）
- 影响的 API：无新端点；无契约变更（仅 MCP 客户端侧错误处理）
- 影响的 DB 表：无
- 影响的编排测试：无（无新 HTTP 端点语义）
- 影响的 smoke 测试：无新增用例（部署后既有套件回归）
- 影响的测试文档：
  - `core-PB-relationship-test-cases.md`（#44 UT/ST、#43 UT/ST）
  - `core-S06-test-cases.md`（#42 UT）

## 部署影响

- 是否需要部署：是
- 部署原因：#43/#44 为前端画布代码变更，需重建 compose 前端镜像生效（mcp-server stdio 进程随客户端分发，不经 compose）
- 影响环境：本地 / 生产（compose）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（纯前端命中/悬浮逻辑 + MCP 客户端错误处理，后向兼容）
- 是否需要 smoke：是（部署后既有 smoke 套件回归，不新增用例）

## 变更概述

1. **#44 关系线命中修复**：`hit_test_reference_tier` 改为「线型同源 + 最近距离优先」——按 `line_type`（bezier/orthogonal）取与绘制一致的几何计算点到线距离，阈值（8px）内取距离最小者；全部超阈值则不命中（空白语义不被远处线劫持）。
2. **#43 悬浮 tooltip**：新增画布 hover 检测（pointermove 未拖拽时），复用 #44 修复后的同一最近命中函数（悬停/点击同口径）；命中时在光标附近渲染 tooltip：`源表.源字段 → 目标表.目标字段`（表维度下至少 `源表 → 目标表`）；悬停不改变选中态，点击选中逻辑不变。
3. **#42 MCP 写入诊断与回退**：①`ApiClient::request` 非 2xx 且 body 非 JSON 时，错误 details 携带 `http_status` / `content_type` / `body_excerpt`（截断 200 字符），不再仅「非 JSON」；②409 时即使 body 非 JSON，响应文本含 `USE_OP_CHANNEL` 字样也走 WS op 通道回退；③`import_schema` tool 描述明确「仅新建，不覆盖已有 id」。
