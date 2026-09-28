# 实现任务：fix-issues-34-35-header-comment-lod-anchor

> 关联 GitHub OPEN issue #34（表头注释渐变亮段对比度）与 #35（拓扑档注释缺失 + 关系线字段维度锚定）。
> 每批闭环：业务代码 + UT/ST + OpenLogos reporter。
> 部署决策：不需要部署（无 `[deploy]` section，与 proposal.md 一致）。

## [delta] D1 规格修订

- [x] D1.1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — MODIFIED R-CMT-CONTRAST 段：#34 渐变双端点 worst-case 判据明文化（两端同时 ≥4.5:1 才免 chip）
- [x] D1.2 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5.12 ADDED R-LOD-08 拓扑档关系线表级锚定（表头中线 × 左右缘端口，选侧沿用 §3.4）；R-LOD-02 注释渲染补实现注记
- [x] D1.3 产出 delta `deltas/test/core-PE-design-system-test-cases.md` — MODIFIED UT-PE-CMT-01（双端点断言 + 橙/粉/紫/棕表头 × 亮/暗主题）
- [x] D1.4 产出 delta `deltas/test/core-CR-canvas-test-cases.md` — MODIFIED UT-CR-LOD-01 / ST-CR-LOD-01（拓扑档注释可见 + 表级锚点汇聚断言）

## [delta] D2 规格自检

- [x] D2.1 从磁盘读回本变更全部 Markdown delta 片段，向用户展示实际原文

## [code] C1 #34 表头注释双端点对比度

- [ ] C1.1 列出本批覆盖 UT/ST ID（UT-PE-CMT-01）并实现：`comment_foreground` 双端点 worst-case（满 tint 合成端 + 纯表体端同时 ≥4.5:1；chip 兜底双端合成取最优）
- [ ] C1.2 更新 UT-PE-CMT-01（橙/粉/紫/棕表头 × 亮/暗主题双端断言）；同批 OpenLogos reporter

## [code] C2 #35 拓扑档注释 + 表级锚定

- [ ] C2.1 列出本批覆盖 UT/ST ID（UT-CR-LOD-01 / ST-CR-LOD-01）并实现：`draw_table_topology_body` 渲染注释副文本（0.8× 拓扑字号、C1 同决策前景、截断）；`calc_path`/`calc_orthogonal_path`/`calc_straight_path` 增 `tier` 入参，拓扑档锚点 = 表头中线 × 左右缘；`draw_relation`/`draw_canvas` 透传；lod 探针暴露锚定模式与注释计数
- [ ] C2.2 对齐 `fix_issues_19_22_ut.rs` / `canvas_path_resize_ghost_ut.rs` 的 calc_* 签名；更新 UT-CR-LOD-01 / ST-CR-LOD-01 断言；同批 OpenLogos reporter

## [follow-up] 关闭 issue 与归档

- [ ] F1 verify PASS 后带验收评论回复并关闭 GitHub issue #34、#35（引用提交/批次）
- [ ] F2 用户授权后 `openlogos archive fix-issues-34-35-header-comment-lod-anchor`，随后确认 git push

## 人类确认点

- [ ] H1 用户确认本提案后，才开始产出 delta（D1～D2）（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权）
- [ ] H2 delta 完成后，等待用户明确授权 `openlogos merge`（/goal 授权）
- [ ] H3 merge 完成后自动提交规格文档，并按合并规格分批实现（C1～C2），每批完成后自动提交代码
- [ ] H4 实现完成后，等待用户明确授权 `openlogos verify`（/goal 授权）
- [ ] H5 verify PASS 后，等待用户明确授权 `openlogos archive`（/goal 授权）
- [ ] H6 归档提交完成后，询问用户是否执行 `git push`（/goal 授权）
