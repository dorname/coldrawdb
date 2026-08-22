# 实现任务

> module: core | proposal: optimize-canvas-connect-and-drag

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §3/§3.1：拖字段出线为主、点击两点为辅、橡皮筋、松手进确认条
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §3 手势 + 拖对象时 rAF 跟线、网格仅 pointerup 对齐
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — §1.2 移动：拖动中实时跟随，松手网格对齐
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 橡皮筋出线、拖表每帧更新 SVG 路径、松手 12px 对齐
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增拖字段出线 UT/ST，保留 ST-PB-01 点击两点
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增拖表跟线 / 松手对齐用例
- [x] 产出 delta 文件到 `deltas/test/core-PU-unified-prototype-test-cases.md` — ST-PU-06 保留点击；新增拖出线与拖表跟线

## [code] 代码实现

- [ ] 实现代码变更