# 实现任务

> module: core | proposal: implement-unified-prototype-spec-parity

## 执行约束

- 唯一现行视觉与交互基线：`core-01-editor-prototype.html`。历史 S03/S04/S05 独立原型仅作差异参考。
- 不改已合并 API、DDL、后端语义与主原型 HTML。发现契约缺口时先写差异，不发明端点。
- 每批必须同时包含：业务代码 + 对应 UT/ST/e2e + OpenLogos reporter。输出代码前列出本批用例 ID。
- 落地一个 skip 用例后，立即从 `SPEC_PARITY_SKIP_IDS` 删除该 ID。
- `?share=` 匿名只读、S01 保存/409、IO、命令面板、设计系统、S06 MCP 不可回退。
- 主原型演示器控件不要求生产原样提供，也不得因此勾选验收完成。

## [delta] 实现与测试状态回写

- [x] 产出 delta 文件到 `deltas/reference/implementation/core-frontend-alignment-acceptance.md` — §4 verify slug 改为本提案；§7 标明由本提案逐项勾选
- [x] 产出 delta 文件到 `deltas/reference/implementation/core-implementation-checklist.md` — §13 第二阶段执行入口改为本提案，列出 A～D 批次
- [x] 产出 delta 文件到 `deltas/test/core-S01-test-cases.md` — 将「仅规格收口 / 待第二阶段」改为本提案实现
- [x] 产出 delta 文件到 `deltas/test/core-S02-test-cases.md` — 同上
- [x] 产出 delta 文件到 `deltas/test/core-S03-test-cases.md` — 同上
- [x] 产出 delta 文件到 `deltas/test/core-S04-test-cases.md` — 同上
- [x] 产出 delta 文件到 `deltas/test/core-S05-test-cases.md` — 同上
- [x] 产出 delta 文件到 `deltas/test/core-PU-unified-prototype-test-cases.md` — ST-PU-22～26 改为本提案实现
- [x] 产出 delta 文件到 `deltas/test/core-V2-production-frontend-test-cases.md` — ST-FE-ALIGN-01～04 改为本提案实现
- [x] 产出 delta 文件到 `deltas/test/core-KB-shortcut-test-cases.md` — ST-KB-* 改为本提案实现
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — ST-PC-MENU/FMT/INSPECTOR 改为本提案实现

## [code] 代码实现

- [ ] 实现代码变更