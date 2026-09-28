# Delta — core-PE-design-system-test-cases.md（新文件）

> merge 时作为新文件写入 `logos/resources/test/core-PE-design-system-test-cases.md`

> 模块：core | 提案：redesign-phase-e-design-system-migration（Phase E 全批次）
> 路径：`logos/resources/test/core-PE-design-system-test-cases.md`
> 关联规格：core-07 / core-08 / core-09 / core-0a / core-0b / core-0c
> 最后更新：2026-06-15

# Phase E Design System 测试用例

## 1. 测试覆盖矩阵

| 批次 | 单元测试 | e2e | 视觉回归 |
|---|---|---|---|
| E1 Tokens | UT-E1-01..02 | — | — |
| E2 Icons | UT-E2-01..02 | — | — |
| E3 Components | UT-E3-01..08 | — | HP-01..05 |
| E4 CodeView | UT-E4-01..05 | ST-PE-08 | — |
| E5 Dark Mode | UT-E5-01..05 | ST-PE-06 | — |
| E6 Motion | UT-E6-01..04 | ST-PE-07 | — |
| 综合 | — | ST-PE-01..05（Phase A 既有） | — |

## 2. E1 Token 单元测试

### UT-E1-01 — token 命名规范

**目的**：验证所有 E1 扩展 token 以 `--cdb-` 前缀且语义清晰

**步骤**：
1. 解析 `frontend-rs/src/styles.css` 中 `:root` 块
2. 提取所有 `--cdb-*` 自定义属性
3. 断言：数量 ≥ 100
4. 断言：所有 token 名符合 `^--cdb-[a-z]+(-[a-z]+)*(-[0-9]+)?$`
5. 断言：所有 token 值非空

**预期**：100+ token 全部符合规范

### UT-E1-02 — dark mode 占位类存在

**步骤**：
1. 解析 `styles.css` 末尾
2. 断言：含 `[data-mode="dark"]` 选择器
3. 断言：含 `@media (prefers-color-scheme: dark)` 媒体查询

**预期**：E5 占位接口存在

## 3. E2 Icon 单元测试

### UT-E2-01 — 50 图标注册

**步骤**：
1. 解析 `frontend-rs/src/icons.rs`
2. 用 `syn` 解析所有 `pub fn icon_*` 函数
3. 断言：函数数量 ≥ 50
4. 断言：每个函数返回 `impl IntoView`
5. 断言：所有函数在 `pub use` 列表中导出

**预期**：50+ 图标函数全部注册

### UT-E2-02 — 尺寸参数化

**步骤**：
1. 在 Leptos 渲染 `<IconAddTable size=24 />`
2. 断言：渲染的 `<svg width="24" height="24">`
3. 断言：stroke-width 默认为 1.5
4. 断言：stroke="currentColor"

**预期**：尺寸/颜色参数化生效

## 4. E3 Component 单元测试

### UT-E3-01..08 — 八组件渲染 + 交互

每个组件至少 2 个测试：

| UT | 组件 | 测试 1（渲染） | 测试 2（交互） |
|---|---|---|---|
| UT-E3-01 | Button | 渲染 4 个 variant 各 1 个 | 点击触发 on_click |
| UT-E3-02 | Modal | 渲染 4 个 width 各 1 个 | ESC 关闭触发 on_cancel |
| UT-E3-03 | Dropdown | 渲染 trigger=Click/hover | 点击 DropdownItem 触发 on_click |
| UT-E3-04 | Tooltip | 渲染 4 个 placement | hover 200ms 后显示 |
| UT-E3-05 | Popover | 渲染 trigger=Click | 点击 Popover 内部不关闭 |
| UT-E3-06 | Tag | 渲染 6 个 color | closable=true 显示 × |
| UT-E3-07 | Collapse | 渲染多个 Panel | 点击 header 展开/收起 |
| UT-E3-08 | SideSheet | 渲染 placement=Right | mask 关闭触发 on_cancel |

**通用断言**：
- 组件渲染时 `data-testid` 属性存在
- token 引用全部 `var(--cdb-*)`（无硬编码颜色）
- 键盘可达：Tab + Enter/Space 触发

## 5. E4 CodeView 单元测试

### UT-E4-01 — Monaco mount

**步骤**：
1. 渲染 `<CodeView visible=true language=Sql />`
2. mock `monaco.editor.create()`
3. 断言：`monaco.editor.create()` 被调用 1 次
4. 断言：传入的 options.language === `"sql"`
5. 断言：传入的 container 是 `<div class="cdb-monaco-container">`

**预期**：Monaco 在 CodeView 挂载时初始化

### UT-E4-02 — DBML 注入

**步骤**：
1. language=Dbml
2. 断言：`monaco.languages.register("dbml")` 被调用
3. 断言：`monaco.languages.set_monarch_tokens_provider("dbml", ...)` 被调用
4. 断言：DBML token provider 包含至少 5 个关键字高亮规则

**预期**：DBML 语法注入生效

### UT-E4-03 — 复制回调

**步骤**：
1. mock `navigator.clipboard.write_text()`
2. 点击 `<Button class="cdb-code-view__copy">`
3. 断言：`navigator.clipboard.write_text()` 被调用，参数为当前 Monaco value
4. 断言：toast.success("已复制到剪贴板") 被调用

**预期**：复制按钮工作

### UT-E4-04 — 销毁时清理

**步骤**：
1. 渲染 CodeView → 卸载
2. 断言：调用了 `monaco.editor.dispose()` 或类似清理

**预期**：内存泄漏防护

### UT-E4-05 — Monaco 主题同步

**步骤**：
1. 渲染 CodeView with `ThemeMode::Light` → 断言 `set_theme("vs")`
2. 切换 `ThemeMode::Dark` → 断言 `set_theme("vs-dark")`

**预期**：主题切换实时更新 Monaco

## 6. E5 Dark Mode 单元测试

### UT-E5-01 — token 切换

**步骤**：
1. 初始 `<html>` 无 `data-mode` 属性
2. 调用 `THEME_MODE.set(ThemeMode::Dark)`
3. 断言：`<html data-mode="dark">`
4. 断言：`getComputedStyle(:root).getPropertyValue("--cdb-color-bg-0") === "#16161a"`

**预期**：暗色 token 切换生效

### UT-E5-02 — 持久化

**步骤**：
1. `THEME_MODE.set(ThemeMode::Dark)`
2. 断言：`localStorage.getItem("cdb-mode") === "dark"`
3. 刷新页面（mock `localStorage`）
4. 断言：`THEME_MODE.get() === ThemeMode::Dark`

**预期**：选择持久化

### UT-E5-03 — 跟随系统

**步骤**：
1. `THEME_MODE.set(ThemeMode::System)`
2. mock `matchMedia("(prefers-color-scheme: dark)").matches === true`
3. 断言：`<html data-mode="dark">`
4. mock 媒体查询变化为 false
5. 断言：`<html data-mode="light">`

**预期**：System 模式响应媒体查询

### UT-E5-04 — Monaco 主题（合并到 UT-E4-05）

### UT-E5-05 — 跨标签页同步

**步骤**：
1. Tab A 设置 `ThemeMode::Dark` → `localStorage["cdb-mode"] = "dark"`
2. Tab B 触发 `storage` 事件 `{ key: "cdb-mode", newValue: "dark" }`
3. 断言：Tab B `THEME_MODE.get() === ThemeMode::Dark`

**预期**：跨标签页同步

## 7. E6 Motion 单元测试

### UT-E6-01 — Modal 关闭动画

**步骤**：
1. 渲染 Modal visible=true
2. 触发 close
3. 断言：200ms 内元素保留（动画播放）
4. 断言：200ms 后元素卸载
5. 断言：CSS `animation` 包含 `cdb-fade-out`

**预期**：退出动画正确播放

### UT-E6-02 — SideSheet slide-out

**步骤**：
1. 渲染 SideSheet
2. 触发 close
3. 断言：CSS `animation` 包含 `cdb-slide-out-right`

**预期**：slide-out 动画

### UT-E6-03 — Issues 徽章 pulse

**步骤**：
1. 设置 issues count = 5
2. 渲染 Issues 折叠面板
3. 断言：Tag 元素 `animation: cdb-pulse 2s ease-in-out infinite`
4. 设置 count = 0
5. 断言：动画 `animation: none`

**预期**：count > 0 时 pulse

### UT-E6-04 — reduced-motion

**步骤**：
1. mock `matchMedia("(prefers-reduced-motion: reduce)").matches === true`
2. 渲染 Modal
3. 断言：所有 `animation-duration: 0.01ms`
4. 断言：所有 `transition-duration: 0.01ms`

**预期**：减弱模式生效

## 7.5 R6 按钮与面板动效（UT-R6）

### UT-R6-01 — 保存圆点 pulse 命名拆分

**步骤**：
1. 读取 `frontend-rs/src/styles.css`
2. 断言：存在 `@keyframes cdb-pulse-opacity`
3. 断言：`.cdb-save-dot--saving` 引用 `cdb-pulse-opacity`
4. 断言：`@keyframes cdb-pulse` 仅定义 `transform: scale`（无 opacity 版重复）

**预期**：保存态与 Issues 徽章 pulse 互不覆盖

### UT-R6-02 — 按钮 focus / primary active

**步骤**：
1. 断言：`.cdb-btn:focus-visible` 含 `var(--cdb-shadow-focus)`
2. 断言：`.cdb-btn--primary:active` 含 `var(--cdb-color-primary-active)`

**预期**：键盘焦点与主按钮按压态可验收

### UT-R6-03 — 面板 spring 入场

**步骤**：
1. 断言：`.cdb-inspector` animation 含 `var(--cdb-easing-spring)`
2. 断言：`.cdb-has-io-drawer .cdb-io-drawer` animation 含 `var(--cdb-easing-spring)`
3. 断言：`.cdb-app-bar__overflow-menu` animation 含 `var(--cdb-easing-spring)`

**预期**：三处面板使用 spring easing

## 8. e2e 测试（Playwright）

### ST-PE-01..05 — Phase A 视觉回归（HP-01..05 选择器适配）

E2/E3 后以下选择器需更新（来自 Phase A 既有用例）：

| 旧选择器 | 新选择器 |
|---|---|
| `.cdb-tool-rail-add-table` (unicode `+`) | `.cdb-tool-rail-add-table > svg` (E2 Icon) |
| `.cdb-issues-badge` 文本 | `.cdb-issues-collapse .cdb-tag--warning` (E3 Tag) |
| `.cdb-modal-new` 容器 | `.cdb-modal-new[data-testid=cdb-modal-new]` (E3 Modal) |
| `.cdb-app-bar button[title="撤销"]` | `.cdb-app-bar [data-testid=btn-undo]` (E3 Button) |
| `.cdb-field-type` 文本 | `.cdb-field-type > .cdb-tag` (E3 Tag) |

### ST-PE-06 — 暗色模式切换

**步骤**：
1. 访问 `/editor`
2. 点击 `btn-theme-toggle`
3. 截图：浅色
4. 再点击 `btn-theme-toggle`
5. 截图：暗色
6. 断言：截图差异 > 30%（像素级 diff）
7. 验证 `data-mode` 属性在 DOM 上

### ST-PE-07 — 模态动效

**步骤**：
1. 打开 New 模态
2. 等待 200ms（动画完成）
3. 截图：模态入场结束态
4. 关闭模态
5. 在 100ms 时截图（动画进行中）
6. 在 300ms 时截图（动画完成）
7. 断言：模态在 300ms 时已从 DOM 移除

### ST-PE-08 — Monaco 加载

**步骤**：
1. 访问 `/editor`，等待初始加载
2. 断言：network 面板无 `monaco-editor` 请求（lazy load）
3. 点击 `btn-code-view`
4. 等待 monaco bundle 下载完成
5. 断言：出现 `monaco-editor` 请求
6. 断言：CodeView 显示 SQL 文本
7. 点击复制按钮
8. 断言：剪贴板内容 = SQL 文本

## 9. OpenLogos Reporter 格式

每个测试完成后写入 `logos/resources/verify/test-results.jsonl`：

```jsonl
{"id": "UT-E1-01", "module": "core", "result": "PASS", "duration_ms": 12, "ts": "2026-06-15T..."}
{"id": "UT-E2-01", "module": "core", "result": "PASS", "duration_ms": 8, "ts": "2026-06-15T..."}
...
{"id": "ST-PE-08", "module": "core", "result": "PASS", "duration_ms": 4500, "ts": "2026-06-15T..."}
```

## 10. 验收总览

- UT-E1..E6 共 26 个单元测试全 PASS
- ST-PE-01..08 共 8 个 e2e 测试全 PASS
- HP-01..05 Phase A 回归全 PASS
- OpenLogos reporter 写入 `test-results.jsonl` ≥ 34 条记录
- E1 阶段：token ≥ 100
- E2 阶段：图标函数 ≥ 50
- E3 阶段：8 组件全部落地
- E4 阶段：CodeView + CommandPalette 工作
- E5 阶段：暗色模式切换 + 持久化工作
- E6 阶段：动效在 8 组件上工作

## 统一原型对齐范围与状态

Design System：token / icon / 组件 / dark / motion，须与主原型壳层（auth/rooms/room-editor）一致。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED — 对齐合同

| ID | 变更 | 合同 |
|---|---|---|
| UT-E1-01/02 | MODIFIED | token 在 auth/rooms/editor 三页可读；`:root` 与 `[data-mode=dark]` 完整 |
| UT-E2-* | 保留 | 图标注册；关键工具/状态图标在主链可见 |
| UT-E3-* | MODIFIED | Button/Modal/Drawer/Tag 等用于统一壳层；关闭行为无残留 |
| UT-E5-* / ST-PE-06 | MODIFIED | 主题切换覆盖 auth→rooms→editor；刷新保持策略按规格 |
| UT-E6-* / ST-PE-07 | MODIFIED | Toast/抽屉/光标 motion；`prefers-reduced-motion` 降级 |
| ST-PE-SHELL-01（ADDED） | ADDED | 主原型视觉基线对照：间距/圆角/玻璃层级不回退到历史独立原型风格 |
| ST-PE-CONTRAST-01（ADDED） | ADDED | 暗色下 StatusBar `ws-status`/`ot-rev` 与 AppBar 保存态可读 |

## 第二阶段说明

像素级视觉回归与全量 HP 截图：规格合同已立；执行与对比基线更新标为**待第二阶段**。

---

## 合并自 fix-remote-github-issues（2026-09-15）

## MODIFIED — UT-E5-02 — 持久化（强制生产接线）

### UT-E5-02 — 主题写入 `localStorage["cdb-mode"]`

- **位置**：`frontend-rs/src/editor_panels.rs`（`btn-theme-toggle` 与启动恢复）
- **合同（不变键名）**：`localStorage["cdb-mode"]` ∈ `"light" | "dark" | "system"`
- **步骤 / 断言**：
  1. 点击切换到浅色 → `document.documentElement.dataset.mode === "light"`
  2. `localStorage.getItem("cdb-mode") === "light"`
  3. 再切回深色 → `cdb-mode === "dark"` 且 `data-mode="dark"`
- **实现缺口关闭**：生产路径不得仅改 DOM attribute；必须与 core-0b §6 同步写读

## ADDED — UT-E5-06 — 启动时从 localStorage 恢复主题

### UT-E5-06 — 刷新后主题保持

- **位置**：`read_html_data_mode` / AppRoot 初始化（或等价 boot 函数）
- **前置**：`localStorage["cdb-mode"]="light"`；`index.html` 默认仍可为 `data-mode="dark"`
- **步骤**：模拟冷启动读取
- **断言**：
  1. 启动后 `data-mode="light"`（localStorage 优先于 html 默认）
  2. `theme_mode` signal == `"light"`
  3. 缺省 / 非法值时回落规格默认（与 core-0b 一致；显式 light/dark 优先于 system）

## MODIFIED — ST-PE-06 — 暗色模式切换含刷新保持

### ST-PE-06 — 暗色模式切换

- 既有：点击 `btn-theme-toggle` 切换 light↔dark
- **ADDED**：切换到 light 后刷新（或重新 mount）→ 仍为 light；`cdb-mode` 仍为 `"light"`

## MODIFIED — 验收总览 / 附录（如有 ID 表）

登记 UT-E5-06；UT-E5-02 状态由「规格占位」改为「本提案必须实现」。

## ID 登记表（OpenLogos ledger）

> `scripts/validate-openlogos-ledger.mjs` 仅识别 `| UT-… |` / `| ST-… |` 表格行。

| ID | 层级 | 说明 |
|---|---|---|
| UT-E5-02 | UT | 主题切换写入 `localStorage["cdb-mode"]` |
| UT-E5-06 | UT | 冷启动从 `cdb-mode` 恢复 `data-mode` / theme_mode |

---

## 合并自 fix-remote-github-issues-7-18（2026-09-18）

## ADDED — UT-PE-AB-01 — 返回按钮 ghost 样式锚点

- **位置**：`frontend-rs/src/styles.css` 断言（`include_str!`）
- **断言**：
  - `.cdb-btn` 或 `.cdb-back-to-rooms` 含 `display: inline-flex` + `align-items: center`（图标文字同行居中）
  - `.cdb-back-to-rooms`（或其对 ghost 的覆盖）`border-color: transparent`；hover 态有独立底色规则
  - 可点击区域高度 ≥ 36px（min-height / height 规则存在）

## ADDED — UT-PE-AB-02 — room-badge 截断规则锚点

- **位置**：`frontend-rs/src/styles.css` + `editor_panels.rs` 锚点
- **断言**：
  - `.cdb-room-badge strong` 选择器存在且含 `overflow: hidden` + `text-overflow: ellipsis` + `white-space: nowrap`
  - `.cdb-room-badge` 保留 `max-width`；AppBar 中部容器含 `min-width: 0`
  - 生产 DOM 中 room-badge 按钮带 `title`（全文兜底）；`diagram-title` 带 `title`

## ADDED — ST-PE-07 — e2e：长房间名截断与邀请按钮可达

- **GIVEN**：加入名称 ≥24 字符的房间，进入其编辑器
- **WHEN**：观察顶栏中部；随后将视口宽度降至 1280
- **THEN**：room-badge 省略号截断且 `title` 为完整房间名；图名不溢出；`btn-invite` 始终可见可点；`btn-back-to-rooms` 图标与「空间」文字同行无堆叠
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S04"`）

## MODIFIED — ID 登记表（OpenLogos ledger）

> `scripts/validate-openlogos-ledger.mjs` 仅识别 `| UT-… |` / `| ST-… |` 表格行。

| ID | 层级 | 说明 |
|---|---|---|
| UT-E5-02 | UT | 主题切换写入 `localStorage["cdb-mode"]` |
| UT-E5-06 | UT | 冷启动从 `cdb-mode` 恢复 `data-mode` / theme_mode |
| UT-PE-AB-01 | UT | 返回按钮 ghost 透明边框 + inline-flex 锚点 |
| UT-PE-AB-02 | UT | room-badge 文本节点截断规则锚点 |
| ST-PE-07 | ST | 长房间名截断 + title 全文 + 邀请可达 |

---

## 合并自 fix-open-issues-26-33（2026-09-27）

## ADDED — UT-PE-HL-01 — 选中高亮参数纯函数（#31）

- **位置**：`frontend-rs/src/editor_render.rs`（高亮态解析纯函数，如 `relation_render_style(is_related)` / `table_render_alpha(is_related)`）
- **断言**：
  1. 选中一张表后：相关关系线线宽 ≥ 默认 1.5× 且 alpha = 1；非相关关系线 alpha ≤ 0.25；非相关表 alpha ≤ 0.5；邻接表 alpha = 1
  2. 选中一条关系后：该关系及其两端表为相关态；其余按非相关处理
  3. 无选中：全部恢复默认线宽与 alpha = 1
  4. 只读模式下同样的输入产生同样的视觉参数（只读不改变高亮反馈）

## ADDED — UT-PE-CMT-01 — 注释对比度前景计算纯函数（#32）

- **位置**：`frontend-rs/src/editor_render.rs`（注释前景/衬底决策纯函数，如 `comment_foreground(header_segment_bg)` / `needs_comment_chip(contrast)`）
- **断言**：
  1. 注释区有效背景（渐变段色）亮度高于阈值 → 选深色前景对；低于阈值 → 浅色前景对（与 R-COLOR-04 同阈值口径）
  2. 前景对对比度仍 < 4.5:1 → `needs_comment_chip` 返回 true（启用衬底/弱化渐变兜底）；≥ 4.5:1 → false
  3. 蓝 / 绿 / 紫三种预设表头色 × 亮 / 暗主题共 6 组输入，输出前景均满足 ≥ 4.5:1（含兜底生效情形）
  4. 空注释输入不产生任何注释渲染参数（空注释不占位不回归）

## ADDED — ST-PE-10 — e2e：选中高亮与注释可读性目视锚点（#31 / #32）

> 改号注记：原编号 ST-PE-08 与既有「ST-PE-08 — Monaco 加载」（E4 CodeView）撞号，按场景编号全局唯一规则改为 ST-PE-10（ST-PE-09 已由 #33 占用）。

- **位置**：`frontend-rs` e2e（独立 spec 或并入 cr-comment-color.spec）
- **GIVEN**：画布含 ≥3 张表（表头色分别蓝 / 绿 / 紫）、≥3 条关系、表与字段含中文注释；注释显示模式 = 英文名+注释
- **WHEN**：选中一张有 ≥3 条关系的表；随后切换亮 / 暗主题各观察一次
- **THEN**：
  1. 相关连线明显更粗更亮，非相关表/线退到背景层（截图锚点 + 关键 alpha/线宽断言经 `data-testid` 或 canvas 参数探针）
  2. 三种表头色 × 两主题下表头注释均可读（无近白字压浅底 / 深色字压深渐变）
  3. 取消选中后全图恢复正常
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## ADDED — UT-PE-VIS-01 — 语义图标统一族锚点（#33）

- **位置**：`frontend-rs/src/editor_render.rs` / 图标组件 + `include_str!` 锚点
- **断言**：
  1. PK / FK / NOT NULL / UNIQUE 徽章渲染共用统一尺寸（12px）与描边（1.5px）常量（同源引用，无散值）
  2. 徽章色值仅取自设计 token 语义色阶（源码锚点无裸 hex 散值）
  3. ToolRail 图标统一 stroke（1.5px）与尺寸（20px / IconBox 24px）；激活态规则存在（主色描边 + 浅底）
  4. 字块角标渲染路径标记 deprecated 或已移除（不再与图标族混用）

## ADDED — ST-PE-09 — e2e：视觉体系统一目视锚点（#33）

- **位置**：`frontend-rs` e2e（独立 spec）
- **GIVEN**：画布含 ≥2 张表（含 PK / FK / NOT NULL / UNIQUE 字段各一）、≥1 条关系；亮 / 暗主题
- **WHEN**：观察表卡、徽章、关系端点、ToolRail；切换主题再观察
- **THEN**：
  1. 表卡圆角 / 投影 / 表头渐变一致（截图锚点 + 关键 CSS/常量断言）
  2. 徽章图标族风格统一（同为线条图标，无字块角标混搭）；关系端点为统一 crow's foot 族
  3. ToolRail 图标描边/尺寸/激活态一致
  4. 主题切换后无风格漂移
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## MODIFIED — ID 登记表（OpenLogos ledger，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PE-HL-01 | UT | 选中高亮参数纯函数（相关线 ≥1.5× / 非相关线 ≤0.25 / 非相关表 ≤0.5） |
| UT-PE-CMT-01 | UT | 注释对比度前景计算（分区背景 + 4.5:1 + 衬底兜底） |
| ST-PE-10 | ST | 选中高亮与注释可读性 e2e 目视锚点（原 ST-PE-08 改号，避让 Monaco 用例） |
| UT-PE-VIS-01 | UT | 语义图标统一族锚点（尺寸/描边/色板/激活态） |
| ST-PE-09 | ST | 视觉体系统一 e2e 目视锚点 |

## 合并自 fix-31-related-line-emphasis（2026-09-28）

## MODIFIED — UT-PE-HL-01 — 选中高亮参数纯函数（#31；fix-31-related-line-emphasis 提亮）

- **位置**：`frontend-rs/src/editor_render.rs`（高亮态解析纯函数，如 `relation_render_width(base, related)` / `table_render_alpha(...)` + `draw_relation` 相关态分支源码锚点）
- **断言**：
  1. 选中一张表后：相关关系线线宽 = 默认 1.5×（`RELATED_RELATION_WIDTH_FACTOR`）且 alpha = 1；**主色强制 `palette.selected`、光晕 `palette.selected_soft`（8px，选中关系自身 10px）**；非相关关系线 alpha ≤ 0.25；非相关表 alpha ≤ 0.5；邻接表 alpha = 1
  2. 选中一条关系后：该关系为选中态（3.5px 主线 + 10px 光晕），其两端表为相关态；共享端点表的其余关系按相关线渲染；其余按非相关处理
  3. 无选中：全部恢复默认线宽、关系解析色（§4.1/§4.2）与 alpha = 1
  4. 只读模式下同样的输入产生同样的视觉参数（只读不改变高亮反馈）
  5. crow's foot 端点记号与主线同色（选中/相关态不得回落基线色 `stroke`）——`draw_relation` 源码锚点

## MODIFIED — ST-PE-10 — e2e：选中高亮与注释可读性目视锚点（#31 / #32；fix-31-related-line-emphasis 提亮）

> 改号注记：原编号 ST-PE-08 与既有「ST-PE-08 — Monaco 加载」（E4 CodeView）撞号，按场景编号全局唯一规则改为 ST-PE-10（ST-PE-09 已由 #33 占用）。

- **位置**：`frontend-rs` e2e（`scripts/test-spec-parity-d.mjs`）
- **GIVEN**：画布含 ≥3 张表（表头色分别蓝 / 绿 / 紫）、≥3 条关系、表与字段含中文注释；注释显示模式 = 英文名+注释
- **WHEN**：选中一张有 ≥3 条关系的表；随后切换亮 / 暗主题各观察一次
- **THEN**：
  1. 相关连线以**选中色族**亮显并带发光光晕且更粗（`__cdb_hl_probe` 暴露 `related_emphasis` 字段为 true + `rel_width_scale_max` ≥ 1.5），非相关表/线退到背景层（`table_alpha_min` ≤ 0.5）；取消选中后探针复位
  2. 三种表头色 × 两主题下表头注释均可读（无近白字压浅底 / 深色字压深渐变）
  3. 亮 / 暗主题截图锚点存在可分辨差异且相关线区域命中选中色族像素
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）

## MODIFIED — ID 登记表（OpenLogos ledger，更新行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PE-HL-01 | UT | 选中高亮参数纯函数（相关线 1.5× + 选中色族 + 发光 halo / 非相关线 ≤0.25 / 非相关表 ≤0.5 / 端点同色） |
| ST-PE-10 | ST | 选中高亮与注释可读性 e2e 目视锚点（原 ST-PE-08 改号；fix-31 提亮：相关线选中色族探针断言） |
