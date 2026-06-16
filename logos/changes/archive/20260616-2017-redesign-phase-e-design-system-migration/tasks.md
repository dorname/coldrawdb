# 实现任务 — redesign-phase-e-design-system-migration

> Phase E：设计系统迁移 + 主分支视觉对齐。代码实现待 merge 后按 `[code]` section 执行。
> 批次执行顺序：E1 → E2 → E3 → E4 → E5 → E6（依赖：E3/E5 依赖 E1；E4 依赖 E3；E6 独立）
> 单 proposal 单 merge：6 批次一次性合并，verify 阶段统一验收。

## [delta] 规格变更

### E1 — Design Tokens 补全

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — E1 完整设计 token 规格（新文件，211 行 17 sections，13 类 ~100 token）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — MODIFIED §1 z-index 体系扩展（L1–L6）+ §3 暗色模式接口预留

### E2 — SVG 图标库

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md` — E2 图标库规格（新文件，~50 图标分 8 类，含命名规范/尺寸/颜色继承）

### E3 — 核心组件重写

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md` — E3 八核心组件规格（Button/Modal/Dropdown/Tooltip/Popover/Tag/Collapse/SideSheet）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — MODIFIED §1 Tool Rail 5 按钮（E2 图标）+ §8 Issues 升级 E3 Collapse + §2–§7 废弃说明
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — MODIFIED §1 AppBar 视觉（E3 Button+Dropdown+Tooltip，E4 btn-code-view，E5 主题按钮）+ §3 9 模态对齐 E3 Modal + §4.2 布局
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — MODIFIED §5.2 token 引用统一替换、§6.4 Inspector E3 组件对齐（E4 Code View 入口预留）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — MODIFIED §2 字段类型徽章 E2 Tag+Icon（7 类型），§2 主键/外键图标
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — MODIFIED §3.x 关系工具 Tooltip/Popover（E3），§4.x 关系线端点图标（E2）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md` — MODIFIED §1.x 索引徽章（E3 Tag 5 类型），§2.x 枚举 E3 Collapse，§3.x 自定义类型徽章
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — MODIFIED §2.x IO 抽屉升级 E3 SideSheet，§4/§5 复制/下载按钮 E2+E3
- [x] 产出 delta → `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — MODIFIED E1 token + E2 icon + E3 Button/Tag/Collapse，ADDED E4 Code View 占位 + E5 暗色 demo

### E4 — Monaco 代码视图

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md` — E4 Monaco + DBML setup + 复制按钮 + lazy load + Command Palette
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — E4 增量已合并到 E3 delta §6.4（Inspector "Code View" 入口）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — E4 增量已合并到 E3 delta §1（AppBar btn-code-view + View 菜单）
- [x] 产出 delta → `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — E4 增量已合并到 E3 delta（Code View 占位 .cdb-monaco-container）

### E5 — 暗色模式

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md` — E5 暗色模式规格（light→dark 完整 token 映射 + JS 切换 + 持久化 + Monaco 同步）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — E5 增量已合并到 E3 delta §1（AppBar btn-theme-toggle + View → Theme 子菜单）

### E6 — 动效与微交互

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md` — E6 动效规格（8 @keyframes + 7 组件动效接线 + reduced-motion 支持）
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md` — E6 增量已合并到 E3 delta §3 Modal / §9 SideSheet 动效引用

### 测试用例

- [x] 产出 delta → `deltas/test/core-PE-design-system-test-cases.md` — 26 UT（E1..E6）+ 8 ST（PE-01..08）+ OpenLogos reporter 格式

## [code] 代码实现

### E1 — Design Tokens

- [ ] `frontend-rs/src/styles.css` — 扩展 `:root` token：`--cdb-color-primary-{hover,active,disabled,soft}` / `--cdb-color-grey-{0..9}` / `--cdb-shadow-{xs,sm,md,lg,xl}` / `--cdb-duration-{fast,base,slow}` / `--cdb-easing-{in,out,inOut}`
- [ ] 单元测试 `frontend-rs/tests/tokens.rs` — `UT-E1-01` token 命名规范 + `UT-E1-02` dark mode 占位类存在
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter

### E2 — SVG 图标库

- [ ] `frontend-rs/src/icons.rs`（新文件）— `Icon` Leptos 组件 trait + `IconAddTable` / `IconEdit` / `IconDelete` / `IconMore` / `IconUndo` / `IconRedo` / `IconChevronDown` / `IconClose` / `IconSearch` / `IconCode` / `IconExport` / `IconImport` / `IconShare` / `IconKey` / `IconLink` / `IconList` / `IconLayers` / `IconNote` / `IconType` / `IconEnum` / 等 ~50 个 SVG path
- [ ] `frontend-rs/src/editor_panels.rs` / `editor_render.rs` / `editor_core.rs` — 替换 unicode emoji 与占位字符为 `<Icon />` 调用
- [ ] 单元测试 `frontend-rs/tests/icons.rs` — `UT-E2-01` 50 个图标注册 + `UT-E2-02` 尺寸参数化
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter

### E3 — 核心组件

- [ ] `frontend-rs/src/components/button.rs`（新文件）— `<Button variant="primary|secondary|tertiary|warning" size="sm|md|lg" />`
- [ ] `frontend-rs/src/components/modal.rs`（新文件）— `<Modal centered closable={esc,mask,close} bodyStyle={maxHeight} />`（参考 main `SemiUIModal` width=480/640/800/1200）
- [ ] `frontend-rs/src/components/dropdown.rs`（新文件）— `<Dropdown trigger="click|hover" position="bottomLeft" />` + `Dropdown.Menu` / `Dropdown.Item` / `Dropdown.Divider`
- [ ] `frontend-rs/src/components/tooltip.rs`（新文件）— `<Tooltip placement="top|bottom|left|right" />`
- [ ] `frontend-rs/src/components/popover.rs`（新文件）— `<Popover trigger="click|hover" />`（含嵌套 TableInfo 等复杂内容）
- [ ] `frontend-rs/src/components/tag.rs`（新文件）— `<Tag color="primary|success|warning|error" />`
- [ ] `frontend-rs/src/components/collapse.rs`（新文件）— `<Collapse lazyRender keepDOM={false} />` + `Collapse.Panel`
- [ ] `frontend-rs/src/components/sidesheet.rs`（新文件）— `<SideSheet placement="right" />`（替代 IO 抽屉的 `<aside>` 实现，行为不变）
- [ ] `frontend-rs/src/editor_panels.rs` — 重构 AppBar / ToolRail / Inspector / IO 抽屉使用新组件
- [ ] 单元测试 `frontend-rs/tests/components.rs` — `UT-E3-01..08` 八组件渲染 + 交互断言
- [ ] 视觉回归 `frontend-rs/tests/visual/` — Playwright 截图 + diff（HP-01~HP-05 选择器适配）
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter

### E4 — Monaco 代码视图

- [ ] `frontend-rs/Cargo.toml` — 引入 `monaco-editor` + `monaco-editor-wasm` + `wasm-bindgen` 升级
- [ ] `frontend-rs/src/code_view.rs`（新文件）— `<CodeView value language="sql|dbml" showCopy={true} />`：Monaco 包装、DBML setup（参考 main `setUpDBML`）、复制按钮（`absolute right-6 bottom-2 z-10`）、lazy load（首次进入时动态 `import()`）
- [ ] `frontend-rs/src/command_palette.rs`（新文件）— `<CommandPalette />`：`Ctrl+K` / `Cmd+K` 触发、模糊搜索表/区域/枚举/便签/关系/类型、Enter 跳转并选中、Esc 关闭；用 E3 Modal 容器
- [ ] `frontend-rs/src/editor_panels.rs` — 接线 `ViewMode`（Canvas / Code 互斥）、`btn-code-view` 启用、`Ctrl+K` 全局监听
- [ ] `frontend-rs/src/styles.css` — `.cdb-command-palette` / `.cdb-code-view` / `.cdb-monaco-container` 样式
- [ ] 单元测试 `frontend-rs/tests/code_view.rs` — `UT-E4-01..04` Monaco mount / DBML 注入 / 复制回调 / 销毁
- [ ] ST-PE-08 浏览器加载 Playwright e2e
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter

### E5 — 暗色模式

- [ ] `frontend-rs/src/settings.rs`（扩展）— `mode: RwSignal<Light|Dark>`、`localStorage["cdb-mode"]` 持久化、`prefers-color-scheme` 初始检测
- [ ] `frontend-rs/src/styles.css` — `[data-mode="dark"]` token 覆盖（`darkBgTheme = "#16161A"` 来自 main）+ `prefers-color-scheme: dark` 媒体查询
- [ ] `frontend-rs/src/editor_panels.rs` — AppBar 暗色切换按钮（图标 + Tooltip）
- [ ] 单元测试 `frontend-rs/tests/dark_mode.rs` — `UT-E5-01..03` token 切换 / 持久化 / 跨标签页同步
- [ ] ST-PE-06 Playwright e2e（暗色模式切换）
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter

### E6 — 动效与微交互

- [ ] `frontend-rs/src/styles.css` — `@keyframes`（`fade-in` / `slide-in-right` / `pulse`）+ `transition` 工具类（`.cdb-transition-{fast,base,slow}`）
- [ ] `frontend-rs/src/components/modal.rs` / `sidesheet.rs` — 入场 / 出场动画（200ms fade + 300ms slide）
- [ ] `frontend-rs/src/editor_panels.rs` — Issues 徽章 `pulse` 动画（issue 数量 > 0 时）
- [ ] 单元测试 `frontend-rs/tests/motion.rs` — `UT-E6-01..04` 动画类名注入 / 触发条件 / 缓动函数
- [ ] ST-PE-07 Playwright e2e（动画结束状态断言）
- [ ] 写入 `logos/resources/verify/test-results.jsonl` OpenLogos reporter
