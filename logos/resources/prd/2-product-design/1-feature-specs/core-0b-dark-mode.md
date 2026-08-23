# Dark Mode 规格（E5）

## 0. 事实基线

唯一现行主题基线：`core-01-editor-prototype.html`。

- 根节点：`<html lang="zh-CN" data-mode="dark|light">`
- 默认演示为 **dark**；`document.documentElement.dataset.mode = state.theme` 同步
- 主题覆盖 **auth / rooms / room-editor（含 Modal、Drawer、Toast、Code）全页**，禁止仅编辑器局部换肤

## 1. 概述

暗色模式通过 `data-mode` 切换整套表面 token（见 `core-07`）。目标：玻璃态高对比可读、WCAG AA 级正文/控件对比，且三态页面视觉语言一致。

## 2. Token 暗色映射

| Token（主原型） | Dark 事实值 |
|---|---|
| `--bg` | `#050f13` |
| `--bg-deep` | `#08191f` |
| `--surface-solid` | `#10262d` |
| `--text` | `#f2fdfe` |
| `--brand` | `#5ee9dc` |
| `--accent` | `#b9a0ff` |

完整映射以 `core-07` dark 表为准。历史 Semi `darkBgTheme = #16161a` **移除为现行事实**。

## 3. CSS 实现

`[data-mode="dark"]` 与 `@media (prefers-color-scheme: dark)` 内 `:root:not([data-mode="light"]):not([data-mode="dark"])` **必须**包含相同 Token 集合（primary / grey / semantic / text / bg / border / shadow / surface），不得仅覆写部分 Token。

新增 surface Token 暗色映射：

| Token | Dark 值 |
|---|---|
| `--cdb-color-canvas-grid` | `rgba(255, 255, 255, 0.06)` |
| `--cdb-color-inspector-edge` | `rgba(255, 255, 255, 0.04)` |
| `--cdb-color-focus-error` | `rgba(248, 113, 113, 0.2)` |
| `--cdb-color-error-hover-bg` | `rgba(248, 113, 113, 0.12)` |

```css
/* styles.css — 节选 */

[data-mode="dark"] {
  color-scheme: dark;
  --cdb-color-primary: #4ba3c4;
  /* ... 完整 dark token，含 surface 四组 ... */
  --cdb-color-canvas-grid: rgba(255, 255, 255, 0.06);
  --cdb-color-inspector-edge: rgba(255, 255, 255, 0.04);
  --cdb-color-focus-error: rgba(248, 113, 113, 0.2);
  --cdb-color-error-hover-bg: rgba(248, 113, 113, 0.12);
}

@media (prefers-color-scheme: dark) {
  :root:not([data-mode="light"]):not([data-mode="dark"]) {
    color-scheme: dark;
    /* 与 [data-mode="dark"] 相同 token 全集（R3） */
  }
}
```

## 4. JS 切换逻辑

```rust
// frontend-rs/src/settings.rs（扩展）
use wasm_bindgen::prelude::*;

pub enum ThemeMode { Light, Dark, System }

pub static THEME_MODE: Lazy<RwSignal<ThemeMode>> = Lazy::new(|| {
    let initial = read_local_storage("cdb-mode")
        .and_then(|s| match s.as_str() {
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            "system" => Some(ThemeMode::System),
            _ => None,
        })
        .unwrap_or(ThemeMode::System);
    
    let signal = create_rw_signal(initial);
    
    // 立即应用
    apply_theme_mode(signal.get());
    
    // 监听变化
    create_effect(move |_| {
        let mode = signal.get();
        write_local_storage("cdb-mode", &format!("{:?}", mode).to_lowercase());
        apply_theme_mode(mode);
    });
    
    signal
});

pub fn apply_theme_mode(mode: ThemeMode) {
    let html = document().document_element().unwrap();
    let resolved = match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::System => {
            if web_sys::window()
                .unwrap()
                .match_media("(prefers-color-scheme: dark)").unwrap()
                .matches()
            { "dark" } else { "light" }
        }
    };
    html.set_attribute("data-mode", resolved).unwrap();
}
```

## 5. UI 入口

### 5.1 AppBar Theme Toggle 按钮（E3 §1 已实现类名）

```rust
<Button
    class="cdb-btn cdb-btn--tertiary"
    data-testid="btn-theme-toggle"
    on_click=toggle_theme
>
    {move || match THEME_MODE.get() {
        ThemeMode::Light => view! { <IconSun /> },
        ThemeMode::Dark => view! { <IconMoon /> },
        ThemeMode::System => view! { <IconSun /> <IconMoon /> },
    }}
</Button>
```

### 5.2 View → Theme 子菜单

```
View ▼
  └─ Theme ▶
       ├─ ☀ 浅色 (Light)
       ├─ 🌙 暗色 (Dark)
       └─ 💻 跟随系统 (System)
```

用 E3 `<Dropdown trigger=Click>` + `<DropdownItem active=... on_click=set_mode>`。

## 6. 持久化与跨标签页

| 行为 | 实现 |
|---|---|
| 持久化 | `localStorage["cdb-mode"]` = `"light" \| "dark" \| "system"` |
| 跨标签页同步 | `window.addEventListener("storage", cb)` 监听其他标签页的修改 |
| 跟随系统变化 | `matchMedia("(prefers-color-scheme: dark)").addEventListener("change", cb)`（仅在 mode=System 时响应） |

## 7. Monaco 主题同步（E4 衔接）

E4 CodeView 在 `setup_dbml()` 中根据当前 mode 设置 Monaco 主题：
- `ThemeMode::Light` → `monaco.editor.set_theme("vs")`
- `ThemeMode::Dark` → `monaco.editor.set_theme("vs-dark")`

切换主题时调用 `monaco.editor.set_theme()` 实时更新（E4 `UT-E4-05`）。

## 8. 验收约束

- 任意页面切换主题后，无未换肤白块/黑块
- `data-mode` 与可见主题一致
- AA：抽样 Auth 标题/正文、Primary 按钮、Banner 文案、Toast 标题
- Code View / Drawer preview 跟随暗色代码表面
- Monaco（若启用）主题与 `data-mode` 同步

## 9. 不在 E5 范围

- 暗色 + 高对比度（accessibility）模式 — V2+
- 用户自定义主题色（替换 `--cdb-color-primary`）— V2+
- 自动切换（按时间 19:00 切暗色）— V2+

## 全页覆盖范围

| 页面 / 表面 | 要求 |
|---|---|
| auth | 故事区、表单、tabs、primary CTA、错误文案 |
| rooms | nav、房间卡、新建虚线卡、用户菜单 |
| room-editor | AppBar、ToolRail、画布表/关系、Inspector、StatusBar、Banner |
| 浮层 | Modal overlay、Popover、Command、Drawer、Toast |
| 代码 / 预览 | `.code-area` / `.preview` 使用暗色代码底（原型 `#061217`） |

主原型另有 `html[data-mode="dark"] …` 组件级增强（边框改 `--line-strong`、primary 按钮近黑字等），生产须等价实现或收进 token，避免漏表面。

## 全页面主题切换入口

| 入口 | 行为 |
|---|---|
| rooms：主题按钮 | `aria-label="切换主题"`；dark↔light |
| editor：更多菜单 `btn-theme-toggle` | 同上 + Toast「主题已切换」 |
| 偏好设置 Modal | `<select>` 深色玻璃 / 浅色玻璃 |
| 命令面板 | 「切换主题」命令 |

生产可保留 Light / Dark / System 三态与 `localStorage["cdb-mode"]`；**未显式选择时**可跟随 `prefers-color-scheme`，但显式 `data-mode` 优先级更高。

## WCAG AA

- 正文 `--text` 对 `--bg` / 实心表面对比度 ≥ 4.5:1
- 辅助 `--text-2` / `--text-3` 用于非关键文案；关键错误/按钮不得仅用 `--text-3`
- 暗色 primary 按钮：亮 brand 底 + 深字（`#050f13`），避免亮底亮字
- 焦点可见：输入/按钮 focus 使用 brand 边或 focus ring token
- 不依赖「仅色相」区分成功/错误（配合图标与文案）

## 过时主题映射（不再作为现行事实）

- Light→Dark 表中以 `#4ba3c4` / `#16161a` / grey 反转为主的 Semi 映射作为唯一真值
- View 菜单 emoji（☀🌙💻）作为唯一入口文案 → 改用 SVG 图标 + 中文
