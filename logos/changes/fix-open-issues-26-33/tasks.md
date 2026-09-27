# 实现任务：fix-open-issues-26-33

> 关联 GitHub OPEN issue #26～#33。每批闭环：业务代码 + UT/ST + OpenLogos reporter（写入 `logos/resources/verify/test-results.jsonl`）。
> 部署决策：不需要部署（无 `[deploy]` section，与 proposal.md 一致）。

## [delta] D1 #29 导入对称规格

- [x] D1.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md` — import/connect 响应输出 references（fks 展开），序列化结构与字段口径
- [x] D1.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — SQL/数据库两路径对称验收口径（表数、关系数、非空 comment 数一致）
- [x] D1.3 产出 delta `deltas/api/bridge.yaml` — import/connect 响应 schema 增加 references
- [x] D1.4 产出 delta `deltas/test/core-PC-import-export-test-cases.md` — 新增对称用例（同 fixture 双路径出线+注释）；废止/改写 UT-PC-26「import/connect 不产 references」反向断言

## [delta] D2 #27 区域 resize 规格

- [x] D2.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 区域边/角 resize 手柄交互合同、最小尺寸约束、只读禁用、撤销路径
- [x] D2.2 产出 delta `deltas/test/core-CR-canvas-test-cases.md` — 区域 resize UT/ST（手柄拖拽持久化、Inspector 宽高一致、只读禁用、最小尺寸）

## [delta] D3 #28 批量线型规格

- [x] D3.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 「应用到全部关系」入口、批量事务单次 Undo、只读禁用
- [x] D3.2 产出 delta `deltas/api/diagrams.yaml` — 复用既有保存路径的语义澄清（或按需新增批量端点，优先不新增）
- [x] D3.3 产出 delta `deltas/test/core-PB-relationship-test-cases.md` — 批量线型用例（一键切换、刷新保持、单次 Undo、只读禁用）

## [delta] D4 #31/#32 可读性规格

- [x] D4.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 选中高亮落地口径（相关线 ≥1.5× 加粗提亮、非相关表/线 opacity ≤ 0.25）
- [x] D4.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — 注释对比度合同（表头注释前景色自适应/衬底、字号字重、字段注释可读性）
- [x] D4.3 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — 注释对比度与高亮相关 token
- [x] D4.4 产出 delta `deltas/test/core-PE-design-system-test-cases.md` — 对比度与选中高亮用例（蓝/绿/紫表头 × 亮/暗主题）

## [delta] D5 #30 小缩放 LOD 规格

- [x] D5.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — zoom 分档 LOD 渲染合同（阈值、低 zoom 隐藏字段行、关系线加粗）
- [x] D5.2 产出 delta `deltas/test/core-CR-canvas-test-cases.md` — LOD 分档用例（≤50% zoom 表名可读、拓扑可辨）

## [delta] D6 #26 MCP 分批写入规格

- [x] D6.1 产出 delta `deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md` — 分批写入验收条件（超限可分批次写成功、失败 revision 不损坏可重试、文档可见上限指引）
- [x] D6.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 分批写入约定（体积上限、细粒度工具串行 expected_revision、超限错误结构）
- [x] D6.3 产出 delta `deltas/api/mcp-tools.yaml` — update_diagram 体积上限说明、超限错误合同、分批指引写进工具描述
- [x] D6.4 产出 delta `deltas/test/core-S06-test-cases.md` — 分批写入用例（超限错误可诊断、分批串行写成功、失败重试 revision 完好）

## [delta] D7 #33 视觉体系规格

- [x] D7.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — 表卡/关系线/选中态视觉 token 统一
- [x] D7.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md` — PK/FK/NOT NULL/UNIQUE 语义图标与 ToolRail 图标统一口径
- [x] D7.3 产出 delta `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 主原型对齐新视觉体系（覆盖 #27/#28/#30/#31/#32 原型入口）
- [x] D7.4 产出 delta `deltas/test/core-PE-design-system-test-cases.md` — 视觉体系一致性用例

## [delta] D8 规格自检

- [x] D8.1 **验证 API YAML** — `logos/resources/api/` 下所有 delta YAML 必须为有效 YAML 且符合 OpenAPI 3.x 规范（含 `:` 或特殊字符的 description/summary 用双引号包裹）
- [x] D8.2 从磁盘读回本变更全部 Markdown/文本 delta 片段，向用户展示实际原文

## [code] C1 批次 A — #29 导入对称实现

- [ ] C1.1 列出本批覆盖 UT/ST ID（对齐 D1.4）并实现：后端 import/connect 响应序列化 references（`phase3_bridge.rs` / `tables_to_json` / `connect_response_data`）
- [ ] C1.2 前端 `parse_bridge_import_tables` 合并 references 进导入 store；SQL 路径 COMMENT 解析加固（schema 限定名、dump 方言）
- [ ] C1.3 同批补齐 UT/ST + OpenLogos reporter；改写 UT-PC-26 反向断言为正向验收

## [code] C2 批次 B — #27 区域 resize 实现

- [ ] C2.1 列出本批覆盖 UT/ST ID（对齐 D2.2）并实现：区域边/角 resize 手柄命中、拖拽改宽高、落账/撤销与 area_drag 路径一致
- [ ] C2.2 Inspector 增加区域宽高编辑，与画布双向一致；只读模式禁用；最小尺寸约束生效
- [ ] C2.3 同批补齐 UT/ST + OpenLogos reporter

## [code] C3 批次 C — #28 批量线型实现

- [ ] C3.1 列出本批覆盖 UT/ST ID（对齐 D3.3）并实现：Inspector「应用到全部关系」+ 命令面板命令
- [ ] C3.2 批量修改走一次 CommandStack 事务（单次 Undo）；只读禁用；保存刷新保持
- [ ] C3.3 同批补齐 UT/ST + OpenLogos reporter

## [code] C4 批次 D — #31/#32 可读性渲染实现

- [ ] C4.1 列出本批覆盖 UT/ST ID（对齐 D4.4）并实现：选中表相关连线加粗提亮、非相关表/线降透明度（editor_render）
- [ ] C4.2 表头注释前景色按表头色自适应对比度（或半透明衬底）；字号/字重提升；字段注释提亮；亮/暗主题成对
- [ ] C4.3 同批补齐 UT/ST + OpenLogos reporter

## [code] C5 批次 E — #30 小缩放 LOD 实现

- [ ] C5.1 列出本批覆盖 UT/ST ID（对齐 D5.2）并实现：zoom 低于阈值隐藏字段行、保留表名（+注释）、关系线加粗；阈值入 token
- [ ] C5.2 同批补齐 UT/ST + OpenLogos reporter

## [code] C6 批次 F — #26 MCP 分批写入实现

- [ ] C6.1 列出本批覆盖 UT/ST ID（对齐 D6.4）并实现：update_diagram 超限可诊断错误（当前体积、建议分片粒度）
- [ ] C6.2 工具描述与文档写入分批指引与 revision 串行规则；按需裁量是否补 batch 工具（默认不补）
- [ ] C6.3 同批补齐 UT/ST + OpenLogos reporter

## [code] C7 批次 G — #33 视觉体系统一实现

- [ ] C7.1 列出本批覆盖 UT/ST ID（对齐 D7.4）并实现：主原型 + 设计 token 更新先导
- [ ] C7.2 前端 canvas 绘制与图标组件对齐新视觉体系（表卡、语义图标、关系端点、ToolRail）
- [ ] C7.3 同批补齐 UT/ST + OpenLogos reporter

## [follow-up] 关闭 issue 与归档

- [ ] F1 verify PASS 后逐条回复并关闭 GitHub issue #26～#33（引用对应提交/批次）
- [ ] F2 用户授权后 `openlogos archive fix-open-issues-26-33`，随后确认 git push

## 人类确认点

- [ ] H1 用户确认本提案后，才开始产出 delta（D1～D8）
- [ ] H2 delta 完成后，等待用户明确授权 `openlogos merge fix-open-issues-26-33`
- [ ] H3 merge 完成后自动提交规格文档，并按合并规格分批实现（C1～C7），每批完成后自动提交代码
- [ ] H4 实现完成后，等待用户明确授权 `openlogos verify`
- [ ] H5 verify PASS 后，等待用户明确授权 `openlogos archive fix-open-issues-26-33`
- [ ] H6 归档提交完成后，询问用户是否执行 `git push`
