# 实现任务 — wire-editor-canvas

## [code] 批次 1 — 可用性基线
- [x] 接入 `editor_render::Canvas`，移除占位符
- [x] `scripts/start-local.sh`：`env -u NO_COLOR`
- [x] 补全 `styles.css` 缺失类 + 侧栏 flex 链
- [x] `draw_grid` 动态 width/height + CSS 点阵背景

## [code] 批次 2 — drawdb 视觉对齐
- [x] `FloatingControls` 浮动工具栏（Zoom / Undo / Redo）
- [x] 左栏空状态 + 建表/保存移入 Tables Tab
- [x] 顶栏面包屑 + 动态 SaveState
- [x] Toolbar 精简（仅 ↶↷ + 标题 + rev + Share/Export）
- [x] 右栏 Fields Tab 壳 + Footer 动态计数

## [code] 批次 3 — 关键交互
- [x] 标题可编辑 + 失焦保存
- [x] View 菜单 Zoom In/Out/Reset
- [x] Toolbar Share 打开 Share 模态
- [x] Tables 列表字段数 + 色块

## [verify] 验证
- [x] `cargo test -p frontend-rs` UT 通过（52 passed）
- [x] `trunk build` 成功
- [x] Playwright：`hasCanvas` + `floating-controls` + `tables-empty-state` + `cdb-breadcrumb`
