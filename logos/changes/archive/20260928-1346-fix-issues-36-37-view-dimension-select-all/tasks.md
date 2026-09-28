# 实现任务：fix-issues-36-37-view-dimension-select-all

> 关联 GitHub issue #36（维度显式切换）与 #37（Ctrl+A 全选 / Delete 删表可撤销）。
> 部署决策：不需要部署（无 `[deploy]` section，与 proposal.md 一致）。

## [delta] D1 规格修订

- [x] D1.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — MODIFIED §5.12（显式维度模式合同：R-LOD-01 zoom 触发废止、R-VIEW-DIM 三入口/持久化/状态指示、R-LOD-03/04 补偿改挂表维度、R-LOD-08 锚定措辞调整；验收口径改写）+ ADDED §5.13（画布键盘全选与删除：R-KBSEL-01..05）
- [x] D1.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — MODIFIED §15.4：`canvas.lod.tier1-max-zoom` 废止勘误，补偿 token 保留并改注「表维度内 zoom 可读性补偿」
- [x] D1.3 产出 delta `deltas/test/core-CR-canvas-test-cases.md` — MODIFIED UT-CR-LOD-01 / ST-CR-LOD-01（显式维度口径：三入口一致性、维度不随 zoom 串档、补偿仅在表维度）+ 附录 A 行更新
- [x] D1.4 产出 delta `deltas/test/core-KB-shortcut-test-cases.md` — ADDED UT-KB-05（Ctrl/Cmd+A 判定与门控纯函数）/ UT-KB-06（DeleteTable 命令 apply/revert 级联快照）/ ST-KB-SEL-01（全选 + 删除 + 撤销 e2e）+ 附录 A 追加

## [delta] D2 规格自检

- [x] D2.1 从磁盘读回本变更全部 Markdown delta 片段，向用户展示实际原文

## [code] C1 #36 显式维度切换

- [x] C1.1 列出本批覆盖 UT/ST ID（UT-CR-LOD-01 / ST-CR-LOD-01）并实现：`ViewDimension` 枚举 + `VIEW_DIMENSION_STORAGE_KEY` + EditorStore 信号；`draw_canvas` 档位改由显式维度驱动（zoom 仅作表维度内补偿）；ToolRail 维度按钮（激活态）+ 快捷键 `V` + 画布右键菜单切换项 + FloatingControls 状态指示；lod 探针暴露 `view_dimension`
- [x] C1.2 改写 UT-CR-LOD-01 与 ST-CR-LOD-01 断言（显式维度口径）并同批补齐 OpenLogos reporter 记录

## [code] C2 #37 全选与可撤销删除

- [x] C2.1 列出本批覆盖 UT/ST ID（UT-KB-05 / UT-KB-06 / ST-KB-SEL-01）并实现：`Command::DeleteTable { table, references }`（apply/revert 级联快照）+ `on_delete_table` 入栈；快捷键处理器新增 Ctrl/Cmd+A（preventDefault 全选图元）与 Delete/Backspace 表/多选/全选分支；多选删除合并单条撤销单元
- [x] C2.2 编写 UT-KB-05 / UT-KB-06 / ST-KB-SEL-01 并同批补齐 OpenLogos reporter 记录

## [follow-up] 关闭 issue 与归档

- [x] F1 verify PASS 后带验收评论回复并关闭 GitHub issue #36 / #37（引用提交）
- [ ] F2 用户授权后 `openlogos archive fix-issues-36-37-view-dimension-select-all`，随后确认 git push

## 人类确认点

- [x] H1 用户确认本提案后，才开始产出 delta（D1～D2）（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权）
- [x] H2 delta 完成后，等待用户明确授权 `openlogos merge fix-issues-36-37-view-dimension-select-all`（/goal 授权）
- [x] H3 merge 完成后自动提交规格文档，并按合并规格实现（C1、C2 分批闭环），完成后自动提交代码
- [x] H4 实现完成后，等待用户明确授权 `openlogos verify`（/goal 授权）
- [ ] H5 verify PASS 后，等待用户明确授权 `openlogos archive fix-issues-36-37-view-dimension-select-all`（/goal 授权）
- [ ] H6 归档提交完成后，询问用户是否执行 `git push`（/goal 授权）
