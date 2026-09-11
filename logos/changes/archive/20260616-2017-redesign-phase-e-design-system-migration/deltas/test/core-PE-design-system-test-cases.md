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
