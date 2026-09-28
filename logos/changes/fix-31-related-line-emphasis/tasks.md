# 实现任务：fix-31-related-line-emphasis

> 关联 GitHub issue #31（重开）：选中表时相关连线高亮效果不明显。
> 部署决策：不需要部署（无 `[deploy]` section，与 proposal.md 一致）。

## [delta] D1 规格修订

- [x] D1.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — MODIFIED §4.4 密度降噪：相关关系线强制 `palette.selected` 主色 + `selected_soft` 发光 halo（8px），端点记号同色；验收口径更新
- [x] D1.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — MODIFIED §15.5：`related-line-scale` 2.0→1.5 勘误对齐实现；新增 `related-line-color` / `related-line-halo` token 行
- [x] D1.3 产出 delta `deltas/test/core-PE-design-system-test-cases.md` — MODIFIED UT-PE-HL-01 / ST-PE-10 断言口径（相关线选中色族 + 发光 + 端点同色）+ ID 登记表行更新

## [delta] D2 规格自检

- [x] D2.1 从磁盘读回本变更全部 Markdown delta 片段，向用户展示实际原文

## [code] C1 相关连线提亮实现

- [x] C1.1 列出本批覆盖 UT/ST ID（UT-PE-HL-01 / ST-PE-10）并实现：`draw_relation` 相关态（非选中）分支——主色 `palette.selected`、halo `palette.selected_soft` 8px、线宽维持 1.5×；端点记号统一用主线色；`draw_canvas` 透传 related 标记；hl_probe 暴露 related 强调字段
- [x] C1.2 更新 UT-PE-HL-01 断言（选中色族 / 发光 halo / 端点同色锚点）与 ST-PE-10 e2e 断言；同批补齐 OpenLogos reporter 记录

## [follow-up] 关闭 issue 与归档

- [ ] F1 verify PASS 后带验收评论回复并关闭 GitHub issue #31（引用提交）
- [ ] F2 用户授权后 `openlogos archive fix-31-related-line-emphasis`，随后确认 git push

## 人类确认点

- [ ] H1 用户确认本提案后，才开始产出 delta（D1～D2）（/goal「继续拉取issue,完成修复」全程授权）
- [ ] H2 delta 完成后，等待用户明确授权 `openlogos merge fix-31-related-line-emphasis`（/goal 授权）
- [ ] H3 merge 完成后自动提交规格文档，并按合并规格实现（C1），完成后自动提交代码
- [ ] H4 实现完成后，等待用户明确授权 `openlogos verify`（/goal 授权）
- [ ] H5 verify PASS 后，等待用户明确授权 `openlogos archive fix-31-related-line-emphasis`（/goal 授权）
- [ ] H6 归档提交完成后，询问用户是否执行 `git push`（/goal 授权）
